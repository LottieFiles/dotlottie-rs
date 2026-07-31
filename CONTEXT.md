# dotLottie Runtime (dotlottie-rs)

The Rust player runtime for the dotLottie format: renders Lottie animations from `.lottie` packages and executes their interactivity (state machines) and styling (themes) as defined by the dotLottie spec.

## Language

### Styling

**Slot**:
A named, overridable property in a Lottie animation (identified by a slot ID) whose value can be replaced at runtime without editing the animation. Comes in seven types: Color, Scalar, Vector, Position, Gradient, Image, Text.
_Avoid_: property override, placeholder

**Slot ID**:
The unique string identifying a slot within an animation. Theme rules and slot actions target slots by this ID (`slotId` in state machine JSON, `id` in theme rules).

**Theme**:
A JSON document in the `.lottie` package (`t/` directory) containing rules that assign values to slots. Applying a theme writes all of its rules' values to their slots.
_Avoid_: skin, style

**Theme Rule**:
One entry in a theme that targets a single slot with a static value, keyframes, or an expression.

### State machine

**Input**:
A typed, named variable of a state machine (Numeric, String, Boolean, or Event) that guards read and actions write. Set by the host or by actions.
_Avoid_: trigger, variable, parameter

**Input Reference**:
A string-typed action or guard value prefixed with `$` (e.g. `"$level"`) that resolves to the current value of the named input at execution time, instead of being a literal.
_Avoid_: variable substitution, binding

**Global Input**:
A read-only, engine-provided value referenced with the `@` prefix (e.g. `@elapsedTime`). Cannot be written by actions.

**Action**:
A one-shot operation executed by the state machine when a state is entered/exited or an interaction fires. Actions mutate inputs, control the player, or (with this spec update) write slot values.

**Slot Action**:
A state machine action that writes a value to a single slot by slot ID (`SetColorSlot`, `SetScalarSlot`, `SetVectorSlot`, `SetPositionSlot`, `SetTextSlot`, `SetImageSlot`) or restores its authored value (`ResetSlot`). One-shot: it writes once when executed; it does not create a live binding.
_Avoid_: slot binding, data binding

**Authored Value**:
The value a slot has in the animation file as exported, before any theme or slot action writes to it. `ResetSlot` and theme resets restore authored values.
_Avoid_: default value, original value

**Base Value**:
The value a slot falls back to when nothing state-scoped covers it: the active theme's value if a theme rule targets the slot, otherwise the authored value.

**State Slots** *(proposed, separate direction from slot actions)*:
A scoped styling overlay declared on a `PlaybackState` alongside its playback configuration: interpolable slot values that apply while the state is current and release to base values on exit. Values with input references are live-bound — they re-apply when the referenced input changes, unlike slot actions, which are one-shot writes.
_Avoid_: state theme, slot overrides

**Interaction**:
A state machine listener that maps a pointer or playback event (optionally scoped to a layer) to a list of actions.

## Example dialogue

> **Dev**: When the user drags, I want the ball to follow the cursor.
> **Domain expert**: Author a `PointerMove` interaction whose slot action `SetPositionSlot` writes `$pointer_x`/`$pointer_y` input references to the ball's position slot. Each pointer move re-executes the action — there's no live binding; the slot only changes when an action writes it.
> **Dev**: So `SetPositionSlot` is a binding between the input and the slot?
> **Domain expert**: No — call it a slot action, not a binding. It's one write, at execution time. The re-execution comes from the interaction firing repeatedly, not from the slot watching the input.
> **Dev**: And if a theme is applied after my slot action?
> **Domain expert**: Last write wins — a slot holds whatever was written to it most recently, whether by a slot action or a theme rule. Switching animations or resetting the theme restores authored values.
