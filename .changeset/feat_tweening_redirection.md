---
default: minor
---

# feat: state machine tweening redirection

Tweened transitions are now interruptible. A transition arriving while a tween is in
flight retargets it from the currently interpolated pose instead of being ignored until
the tween finishes.

Behavioural changes with no opt-out:

- All tweened transitions are interruptible. Blocking can be recreated with a
  boolean-input guard where it is actually wanted.
- Inputs can be set while tweening; the previous no-op guards on
  `set_numeric` / `set_string` / `set_boolean` are gone.
- The transition pipeline now runs during a tween, evaluating the source state.

`current_frame()` now settles on the target frame once a tween completes, instead of
keeping the frame the tween started from.

Also bumps ThorVG to v1.1.0 for the dynamic tweening API.
