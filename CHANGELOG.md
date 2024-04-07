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
