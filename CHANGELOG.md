## 0.1.33 (2024-10-18)

### Features

#### made thorvg an optional dependency (#248)

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
