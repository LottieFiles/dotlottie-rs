# State Machine Engine: Memory & CPU Optimization Plan

## Context

The state machine engine (`StateMachineEngine`) runs interactively at 60 FPS. `tick(dt)` is called every frame and `post_event()` is called on every pointer interaction. The main hot paths are:

- `run_current_state_pipeline()` — called on every input change / event fire, loops up to 20× per call
- `evaluate_transitions()` — iterates all transitions + guards per state per loop
- `manage_cross_platform_events()` — called on every PointerMove

The code uses a clone-heavy, value-semantics design. Most allocations happen in correct, working logic — the goal is to eliminate *unnecessary* allocations in tight loops without changing behavior.

---

## Critical Files

| File | Role |
|------|------|
| `dotlottie-rs/src/state_machine_engine/mod.rs` | Main engine, ~1437 lines — contains all hot paths |
| `dotlottie-rs/src/state_machine_engine/states/mod.rs` | `StateTrait`, `State` enum |
| `dotlottie-rs/src/state_machine_engine/transitions/mod.rs` | `Transition` enum, `TransitionTrait` |
| `dotlottie-rs/src/state_machine_engine/transitions/guard.rs` | Guard evaluation |
| `dotlottie-rs/src/state_machine_engine/inputs/mod.rs` | `InputManager` |
| `dotlottie-rs/src/state_machine_engine/actions/mod.rs` | Action execution |

---

## Tier 1 — High-Impact Changes

### 1. `detect_cycle()` → `bool`

**Why:** Called on every iteration of the `while tick` loop (up to 20×/pipeline call). Currently allocates a `Vec<String>` + `HashSet` even when no cycle exists (the common case). The single call site only checks `if let Some(_cycle)` — the `Vec<String>` contents are never used.

**Before** (`mod.rs:1028–1048`):
```rust
fn detect_cycle(&self) -> Option<Vec<String>> {
    let mut seen = HashSet::new();
    let mut cycle = Vec::new();
    for state in self.state_history.iter().rev() {
        if !seen.insert(state) {
            let cycle_start = state;
            cycle.push(cycle_start.clone());
            for s in self.state_history.iter().rev() {
                if s == cycle_start { break; }
                cycle.push(s.clone());
            }
            cycle.reverse();
            return Some(cycle);
        }
    }
    None
}
```

**After:**
```rust
fn detect_cycle(&self) -> bool {
    let mut seen = HashSet::new();
    for state in self.state_history.iter() {
        if !seen.insert(state.as_str()) {
            return true;
        }
    }
    false
}
```

Update call site (`mod.rs:931`): `if let Some(_cycle) = self.detect_cycle()` → `if self.detect_cycle()`

---

### 2. `StateTrait::name()` → `&str`

**Why:** Called at 8+ hot-path sites (guard comparisons, history push, state lookup, event observation). Each call currently allocates a `String` unnecessarily. The `name` field is `String` inside the enum — returning a `&str` reference is free.

**Before** (`states/mod.rs:19`):
```rust
fn name(&self) -> String;
// impl:
State::PlaybackState { name, .. } => name.clone(),
State::GlobalState { name, .. } => name.clone(),
```

**After:**
```rust
fn name(&self) -> &str;
// impl:
State::PlaybackState { name, .. } => name,
State::GlobalState { name, .. } => name,
```

**Cascading changes** (compiler-guided — all mechanical):
- `mod.rs:946`: `self.state_history.push(state.name().to_string())` — add back `.to_string()` explicitly (one real allocation, the only necessary one)
- `mod.rs:687`: `observe_on_transition(..., &new_state.name())` — already passes `&str`, works unchanged
- `mod.rs:625,631`: comparisons `global_state.name() == state_name` — `&str == &str`, works unchanged
- `mod.rs:884`: `target_state == self.get_current_state_name()` — works unchanged
- `get_type()` can also be changed to `-> &str` in the same pass

---

### 3. `evaluate_transitions()` — return `MatchedTransition` instead of cloning `Transition`

**Why:** Returns `Option<(String, Transition)>` — clones the entire `Transition` enum (which contains `Option<Vec<Guard>>` with each `Guard` holding String fields) on every matched transition. `set_current_state` only uses the transition for 3 things: `is Tweened?` (pattern match), `.duration() → f32`, `.easing() → [f32; 4]`.

**New struct** (add near `mod.rs` top, after imports):
```rust
struct MatchedTransition {
    target_state: String,
    is_tweened: bool,
    duration: f32,
    easing: [f32; 4],
}
```

**Before** (`mod.rs:865, 873`):
```rust
return Some((target_state.to_string(), transition.clone()));
// ...
Some((target_state.to_string(), guardless_transition?.clone()))
```

**After** (`mod.rs:802`):
```rust
fn evaluate_transitions(
    &self,
    state_to_evaluate: &State,
    event: Option<&String>,
) -> Option<MatchedTransition> {
    // ... same guard logic ...
    if all_guards_satisfied {
        return Some(MatchedTransition {
            target_state: transition.target_state().to_string(),
            is_tweened: matches!(transition, Transition::Tweened { .. }),
            duration: transition.duration(),
            easing: transition.easing(),
        });
    }
    // guardless:
    let gt = guardless_transition?;
    Some(MatchedTransition {
        target_state: gt.target_state().to_string(),
        is_tweened: matches!(gt, Transition::Tweened { .. }),
        duration: gt.duration(),
        easing: gt.easing(),
    })
}
```

**Update `set_current_state` signature** (`mod.rs:677`):
```rust
fn set_current_state(
    &mut self,
    state_name: &str,
    causing_transition: Option<&MatchedTransition>,
    called_from_global: bool,
) -> Result<(), StateMachineEngineError>
```

Update the body — the tweening block at line 709 becomes:
```rust
if let Some(mt) = causing_transition {
    if mt.is_tweened {
        // use mt.duration, mt.easing directly
    }
}
```

Update callers (`mod.rs:889, 986`): destructure the new return type instead of `(String, Transition)`.

---

### 4. Action vec iteration: `.iter().cloned()` not `.clone()`

**Why:** `.extend(vec.clone())` clones the entire `Vec<Action>` into a temporary allocation, then extends. `.extend(vec.iter().cloned())` clones each `Action` directly into the destination, avoiding the intermediate `Vec` heap allocation. Matters most in `manage_cross_platform_events` which is called on every PointerMove.

**Locations** (`mod.rs`): lines 1101, 1138, 1156, 1068, 1074, 1079, 1223, 1234

**Before:**
```rust
actions_to_execute.extend(interaction.get_actions().clone());
actions_to_execute.extend(actions.clone());
```

**After:**
```rust
actions_to_execute.extend(interaction.get_actions().iter().cloned());
actions_to_execute.extend(actions.iter().cloned());
```

---

### 5. Guard string evaluation: avoid first clone on literal path

**Why:** `string_input_is_satisfied` (`guard.rs:111`) always clones `compare_to` upfront, even on the common literal (non-`$`) path where no clone is needed. On the `$`-variable path, it then clones again and discards the first.

**Before** (`guard.rs:111–124`):
```rust
let mut mut_compare_to = compare_to.clone();
if mut_compare_to.starts_with("$") {
    let value = mut_compare_to.trim_start_matches('$');
    let opt_string_value = input.get_string(value);
    if let Some(string_value) = opt_string_value {
        mut_compare_to = string_value.clone();
    } else {
        return false;
    }
}
// use mut_compare_to
```

**After:**
```rust
let resolved: String;
let effective: &str = if compare_to.starts_with('$') {
    let key = compare_to.trim_start_matches('$');
    match input.get_string(key) {
        Some(s) => { resolved = s; &resolved }
        None => return false,
    }
} else {
    compare_to.as_str()  // zero-cost borrow, no allocation on common path
};

match condition_type {
    TransitionGuardConditionType::Equal    => input_value == effective,
    TransitionGuardConditionType::NotEqual => input_value != effective,
    _ => false,
}
```

---

## Tier 2 — Small Cleanups (zero risk, one pass)

| Location | Change |
|---|---|
| `actions/mod.rs:315` | Remove dead `let _ = target.to_lowercase();` |
| `inputs/mod.rs:49` | `InputManager::new()`: remove `inputs.clone()` on empty HashMap, use two separate `HashMap::new()` calls |
| `mod.rs:1147` | `"".to_string()` → `String::new()` (no heap alloc for empty string) |
| `mod.rs` constructor | Pre-allocate `state_history`: `Vec::with_capacity(max_cycle_count.unwrap_or(20))` |
| `mod.rs` event handlers | Pre-allocate `actions_to_execute`: `Vec::with_capacity(4)` to avoid re-growth for typical action counts |

---

## Tier 3 — Future Architectural Work

These are high-value but require broader restructuring:

**A. `current_state: Option<usize>` index into `state_machine.states`**
Replace the cloned `State` value in `current_state` / `tween_transition_target_state` with indices. Eliminates all `State::clone()` calls entirely. Requires careful handling of GlobalState (stored separately from the states Vec).

**B. `InputManager` return references**
`get_string()` / `get_numeric()` / `get_boolean()` currently return owned values cloned from the HashMap. Returning `Option<&str>` / `Option<f32>` (f32 is Copy) eliminates HashMap-read allocations in guard evaluation and action execution.

---

## Implementation Order

Apply as a single PR in this sequence to keep diffs reviewable:

1. **Tier 2 cleanups** — one commit, trivial
2. **`detect_cycle` → `bool`** — one commit
3. **`name()` → `&str`** — one commit (compiler guides all call sites)
4. **`evaluate_transitions` returns `MatchedTransition`** — one commit
5. **Action vec + guard clone fixes** — one commit

---

## Verification

1. **Run existing tests**: `cargo test -p dotlottie-rs` — all 6 `state_machine*.rs` test files must pass
2. **Run benchmarks** before/after: `cargo bench -p dotlottie-rs -- state_machine`
3. **Add a pipeline benchmark** to `benches/benchmarks.rs`:
   - Load a state machine, tick 1000× with alternating `set_numeric_input` calls
   - Measure `post_event(PointerMove)` throughput separately
4. **Confirm no player regression**: run `animation_loop_*` benchmarks
