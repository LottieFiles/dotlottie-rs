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
