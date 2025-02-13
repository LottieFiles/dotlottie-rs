---
default: minor
---

# feat: add animation tweening support

Added new tweening capabilities to smoothly animate between frames using cubic bezier easing:

- Added `tween` method to directly tween between two frames with linear progress:

  ```rust
  animation.tween(10.0, 20.0, 0.5); // Tween from frame 10 to 20 at 50% progress
  ```

- Added `tween_to` method for time-based tweening with custom easing:

  ```rust
  // Tween to frame 20 from the current frame over 2 seconds using custom easing curve
  animation.tween_to(20.0, 2.0, [0.4, 0.0, 0.6, 1.0]);
  ```

- Added `tween_to_marker` method for marker-based tweening with custom easing:

  ```rust
  // Tween to marker "jump" from the current frame over 2 seconds using custom easing curve
  animation.tween_to_marker("jump", 2.0, [0.4, 0.0, 0.6, 1.0]);
  ```

- Added helper methods to manage tween state:
  - `is_tweening()` - Check if animation is currently tweening
  - `tween_update()` - Update tween progress based on elapsed time

The tweening implementation uses cubic bezier interpolation for smooth easing effects. The easing curve is defined by control points P1(x1,y1) and P2(x2,y2), with P0(0,0) and P3(1,1) fixed.
