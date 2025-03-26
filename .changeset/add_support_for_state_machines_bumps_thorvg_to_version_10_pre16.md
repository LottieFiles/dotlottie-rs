---
default: minor
---

# feat: add support for state machines, bumps ThorVG to version 1.0-pre16

Added cross-platform state machine support. State machines allow you to create deep interactive scenarios with Lottie animations. With a single state machine definition file your animations can become interactive across Web, Android and iOS.

To get started you can either load a state machine via a string with it's contents like so:

```rust
const success = animation.stateMachineLoadData("{initial: ...}");
```

Or from a definition file inside your .lottie:

```rust
const success = animation.stateMachineLoad("state_machine_id");
```

The to start it:

```rust
const success = animation.stateMachineStart();
```

You can add a state machine observer like so:

```rust
struct MyStateMachineObserver;

impl StateMachineObserver for MyStateMachineObserver {
    fn on_transition(&self, previous_state: String, new_state: String) {
        println!(
            "[state machine event] on_transition: {} -> {}",
            previous_state, new_state
        );
    }

    ...
}

    let observer: Arc<dyn StateMachineObserver + 'static> = Arc::new(MyStateMachineObserver {});

    animation.state_machine_subscribe(observer.clone());
```


Notes:

- This update also includes support for event observer bindings for WASM. All player events are supported alongside state machine events.

- Bumps ThorVG to v1.0-pre16 for the use of new OBB (Oriented Bounding Box) support.