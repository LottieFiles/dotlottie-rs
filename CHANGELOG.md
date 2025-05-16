## 0.1.40 (2025-04-16)

### Features

#### state machine support

#### feat: add support for state machines, bumps ThorVG to version 1.0-pre19

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

- Bumps ThorVG to v1.0-pre19 for the use of new OBB (Oriented Bounding Box) support.

## 0.1.39 (2025-03-31)

### Features

#### optimize WASM build for size  (#301)

#### upgrade thorvg v1.0-pre14 (#303)

#### add tweening between markers and frames (#294)

#### 🎸 pick up .lot ext from dotLottie files (#308)

#### upgrade thorvg-v1.0-pre18 (#313)

#### feat: add animation tweening support

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

#### feat: update hit detection from simple layer bounding box to OBB (Oriented Bounding Box)

### Fixes

#### 🐛 WASM build fails with nightly toolchain (#300)

#### chore: Update ThorVG to v1.0-pre12

## 0.1.38 (2025-01-24)

### Features

#### update thorvg to v1.0-pre11 (#288)

## 0.1.37 (2025-01-17)

### Features

#### upgrade thorvg v1.0-pre10 (#244)

### Fixes

#### 🐛 memory leak when LottieRenderer is created (#282)

## 0.1.36 (2025-01-03)

### Features

#### upgrade to thorvg v0.15.8 (#277)

## 0.1.35 (2025-01-02)

### Features

#### optimize android/ios release binaries for size (#274)

#### update thorvg to v0.15.7 (#276)

### Fixes

#### thorvg engine init and termination (#265)

## 0.1.34 (2024-12-09)

### Features

#### 🎸 upgrade to thorvg v0.15.6

### Fixes

#### add -Dfile=false flag for thorvg wasm builds to resolve runtime errors related to missing filesystem operations (#267)

#### minimum deployment version of the iOS and MacOSX targets (#269)

## 0.1.33 (2024-11-20)

### Features

#### made thorvg an optional dependency (#248)

#### update thorvg to version 0.15.2 (#255)

#### Update thorvg to v0.15.4 (#262)

#### handling of dotLottie v2 specs  (#254)

### Fixes

#### removed unused lib.rs file (#247)

## 0.1.32 (2024-10-07)

### Features

#### migration to conan for dotlottie-rs (#225)

#### c-api (#237)

#### chore(dependencies): 🤖 upgrade thorvg v0.14.9

#### chore(dependencies): 🤖 upgrade thorvg v0.15.0

## 0.1.31 (2024-09-04)

### Features

#### added GlobalState node type (#215)

#### bump thorvg to v0.14.5 (#216)

#### layer detection api (#217)

#### x86 android support (#218)

#### bump thorvg to v0.14.8

### Fixes

#### thorvg canvas resize problem. (#206)

#### corrected thorvg canvas sync (#222)

## 0.1.30 (2024-08-01)

### Fixes

#### clippy lints (#213)

## 0.1.29 (2024-07-30)

### Fixes

#### deleted extra println (#211)

## 0.1.28 (2024-07-30)

### Fixes

#### post_event return codes (#207)

## 0.1.27 (2024-07-29)

### Features

#### interactivity sync state (#203)

### Fixes

#### iOS build issue caused by meson 1.5.0 (#204)

#### rolled thorvg back to 0.13.8 (#208)

## 0.1.26 (2024-07-12)

### Fixes

#### chore(dependencies): 🤖 upgrade thorvg@0.14.1

## 0.1.25 (2024-07-05)

### Features

#### chore: 🤖 upgrade thorvg@0.14.0

## 0.1.24 (2024-06-27)

### Features

#### 🎸 expose lottie animation original size (#180)

#### added context methods (#191)

#### added load_state_machine_data (#190)

#### chore(wasm): 🤖 reduce WASM binary size

- **WASM Binary Optimization:**

  - Applied the `-Oz` flag with `emcc` for size optimization.
  - Used the compact `emmalloc` allocator.
  - Used the rust nightly toolchain to remove location details and panic string formatting for a smaller binary size.
  - Reduced binary size by ~142 KB (from 1,245,102 bytes to 1,099,243 bytes).

- **JavaScript Glue Optimization:**

  - Enabled the Closure compiler with the `--closure=1` flag.
  - Reduced glue code size by ~36.88 KB (from 67,964 bytes to 30,197 bytes).

### Fixes

#### removed commented out target_arch (#184)

#### iOS bundle minimum supported version on AppStore fix (#185)

#### 🐛 manifest_string() panics when no manifest available (#189)

#### chore(dependencies): 🤖 upgrade uniffi-rs to v0.28.0

## 0.1.23 (2024-06-19)

### Features

#### dotLottie interactivity v0.1 (#166)

## 0.1.22 (2024-06-13)

### Fixes

#### 🐛 incorrect ios build env variable (#177)

## 0.1.21 (2024-06-13)

### Fixes

#### 🐛 prevent inlining of already embedded image assets in .lottie (#173)

#### wrong minimum OS version on binary target (#174)

#### chore(dependencies): 🤖 thorvg v0.13.7, emsdk v3.1.61, uniffi-rs v0.27.3

## 0.1.20 (2024-05-31)

### Features

#### 🎸 segment duration getter (#161)

#### 🎸 add tvg_canvas_set_viewport integration and bindings (#158)

#### chore: 🤖 update ThorVG 0.13.5

#### feat: 🎸 add tvg_canvas_set_viewport integration and bindings

#### feat(playback): 🎸 segment duration getter

### Fixes

#### 🐛 update speed causes animation frame jump (#160)

#### chore: 🤖 update ThorVG 0.13.6

#### fix(playback): 🐛 update speed causes animation frame jump

## 0.1.19 (2024-05-21)

### Features

#### chore: 🤖 Upgrade ThorVG to v0.13.4 (#148)

Release details: [ThorVG v0.13.4](https://github.com/thorvg/thorvg/releases/tag/v0.13.4)

### Fixes

#### perf: 🚀 Optimize frame interpolation by rounding to 3 decimal places (#148)

## 0.1.18 (2024-05-20)

### Features

#### 🎸 active_animation_id() (#125)

#### 🎸 create_default_config ffi (#129)

#### Add `active_theme_id` function (#131)

### Fixes

#### 🐛 unexpected is_complete result for Bounce modes on load (#130)

#### 🐛 invalid embed of image assets (#132)

#### 🐛 failed to reach end frame on frame interpolation enabled (#134)

#### 🐛 play() after set_frame() resets the animation (#135)

#### 🐛 .lottie file load failure with float speed property (#151)

## 0.1.17 (2024-05-02)

### Features

#### 🎸 active_animation_id() (#125)

#### 🎸 create_default_config ffi (#129)

#### Add `active_theme_id` function (#131)

### Fixes

#### 🐛 unexpected is_complete result for Bounce modes on load (#130)

#### 🐛 invalid embed of image assets (#132)

#### 🐛 failed to reach end frame on frame interpolation enabled (#134)

#### 🐛 play() after set_frame() resets the animation (#135)

## 0.1.16 (2024-04-03)

### Features

#### 🎸 markers (#89)

#### 🎸 theming (#81)

#### 🎸 layout config (#93)

#### 🎸 revert loaded theme (#104)

#### updated readme (#105)

### Fixes

#### 🐛 ensure canvas is cleared before loading a new animation (#86)

#### 🐛 segmentation fault when LottieRender is dropped (#91)

#### 🐛 markers emscripten wasm bindings (#96)

#### 🐛 added a guard for is_complete (#97)

#### 🐛 unnecessary extra array wrapper in theme serialization (#98)

#### added thorvg backlink inside readme (#120)

## 0.1.15 (2024-02-22)

### Fixes

#### added load_error event (#75)

#### 🐛 memory access out of range on resize (#76)

#### 🐛 on_loop/on_complete events are not triggered when in Reverse/Forward modes (#77)

#### 🐛 reset playback_state,loop_count,start_time on load (#71)

#### 🐛 pass animation data to tvg_picture_load_data as a valid C string (#78)

#### 🐛 themes structure in manifest file (#65)

#### re-init dotlottie manager when loading from animation_data (#85)

#### build workflow fix for missing symbols (#87)

## 0.1.14 (2024-02-20)

### Fixes

#### added load_error event (#75)

#### 🐛 memory access out of range on resize (#76)

#### 🐛 on_loop/on_complete events are not triggered when in Reverse/Forward modes (#77)

#### 🐛 reset playback_state,loop_count,start_time on load (#71)

#### 🐛 pass animation data to tvg_picture_load_data as a valid C string (#78)

#### 🐛 themes structure in manifest file (#65)

#### re-init dotlottie manager when loading from animation_data (#85)

## 0.1.13 (2024-02-02)

### Features

#### 🎸 add a way to check if the animation is completed (#58)

#### 🎸 add events support (#46)

#### 🎸 add unsubscribe method (#64)

#### removed manifest setting loading (#67)

### Fixes

#### 🐛 resume playing from the current_frame after pause (#55)

#### 🐛 start_time not updated on set_frame (#56)

#### 🐛 direction change while animation is playing (#60)

#### 🐛 resize method to validate and update the width,height (#57)

#### 🐛 events interface issue in foreign language (#62)

#### 🐛 uniffi-bindgen-cpp doesn't support WithForeign (#63)

#### 🐛 missing [byRef] udl syntax in unsubscribe method (#66)

#### 🐛 rwlock read lock would result in deadlock (#68)

#### 🐛 only pause if animation is playing (#69)

#### 🐛 set_frame to return an error for invalid frame number (#70)

## 0.1.12 (2024-01-22)

### Features

#### 🎸 bounce modes (#41)

#### 🎸 segments (#45)

#### 🎸 background color (#48)

#### 🎸 emscripten bindings .d.ts module generation  (#49)

#### added crate to manage .lotties (#23)

### Fixes

#### 🤖 update emscripten bindings (#40)

## 0.1.11 (2024-01-15)

### Features

#### 🎸 init playback controls implementation (#31)
