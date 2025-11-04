#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


#define DOTLOTTIE_ERROR 1

#define DOTLOTTIE_INVALID_PARAMETER 2

#define DOTLOTTIE_MANIFEST_NOT_AVAILABLE 3

#define DOTLOTTIE_MAX_STR_LENGTH 512

#define DOTLOTTIE_SUCCESS 0

#define LISTENER_TYPE_POINTER_DOWN (1 << 1)

#define LISTENER_TYPE_POINTER_ENTER (1 << 2)

#define LISTENER_TYPE_POINTER_EXIT (1 << 3)

#define LISTENER_TYPE_POINTER_MOVE (1 << 4)

#define LISTENER_TYPE_POINTER_UP (1 << 0)

#define LISTENER_TYPE_UNSET 0

typedef enum DotLottieFit {
  Contain,
  Fill,
  Cover,
  FitWidth,
  FitHeight,
  Void,
} DotLottieFit;

typedef enum Mode {
  Forward,
  Reverse,
  Bounce,
  ReverseBounce,
} Mode;

typedef struct DotLottiePlayer DotLottiePlayer;

typedef struct DotLottieLayout {
  enum DotLottieFit fit;
  float align_x;
  float align_y;
} DotLottieLayout;

typedef struct DotLottieString {
  char value[DOTLOTTIE_MAX_STR_LENGTH];
} DotLottieString;

typedef struct DotLottieConfig {
  enum Mode mode;
  bool loop_animation;
  uint32_t loop_count;
  float speed;
  bool use_frame_interpolation;
  bool autoplay;
  float segment_start;
  float segment_end;
  uint32_t background_color;
  struct DotLottieLayout layout;
  struct DotLottieString marker;
  struct DotLottieString theme_id;
  struct DotLottieString state_machine_id;
  struct DotLottieString animation_id;
} DotLottieConfig;

typedef struct LayerBoundingBox {
  float x1;
  float y1;
  float x2;
  float y2;
  float x3;
  float y3;
  float x4;
  float y4;
} LayerBoundingBox;

typedef struct DotLottieOption_DotLottieString {
  struct DotLottieString value;
  bool defined;
} DotLottieOption_DotLottieString;

typedef struct DotLottieManifest {
  struct DotLottieOption_DotLottieString generator;
  struct DotLottieOption_DotLottieString version;
} DotLottieManifest;

typedef struct DotLottieManifestAnimation {
  struct DotLottieOption_DotLottieString id;
  struct DotLottieOption_DotLottieString name;
  struct DotLottieOption_DotLottieString initial_theme;
  struct DotLottieOption_DotLottieString background;
} DotLottieManifestAnimation;

typedef struct DotLottieManifestStateMachine {
  struct DotLottieString id;
  struct DotLottieOption_DotLottieString name;
} DotLottieManifestStateMachine;

typedef struct DotLottieManifestTheme {
  struct DotLottieString id;
  struct DotLottieOption_DotLottieString name;
} DotLottieManifestTheme;

typedef struct DotLottieMarker {
  struct DotLottieString name;
  float duration;
  float time;
} DotLottieMarker;

typedef void (*OnMessageOp)(const char*);

typedef struct StateMachineInternalObserver {
  OnMessageOp on_message_op;
} StateMachineInternalObserver;

typedef enum DotLottieEvent_Tag {
  PointerDown,
  PointerUp,
  PointerMove,
  PointerEnter,
  PointerExit,
  Click,
  OnComplete,
  OnLoopComplete,
} DotLottieEvent_Tag;

typedef struct PointerDown_Body {
  float x;
  float y;
} PointerDown_Body;

typedef struct PointerUp_Body {
  float x;
  float y;
} PointerUp_Body;

typedef struct PointerMove_Body {
  float x;
  float y;
} PointerMove_Body;

typedef struct PointerEnter_Body {
  float x;
  float y;
} PointerEnter_Body;

typedef struct PointerExit_Body {
  float x;
  float y;
} PointerExit_Body;

typedef struct Click_Body {
  float x;
  float y;
} Click_Body;

typedef struct DotLottieEvent {
  DotLottieEvent_Tag tag;
  union {
    PointerDown_Body pointer_down;
    PointerUp_Body pointer_up;
    PointerMove_Body pointer_move;
    PointerEnter_Body pointer_enter;
    PointerExit_Body pointer_exit;
    Click_Body click;
  };
} DotLottieEvent;

typedef void (*OnTransitionOp)(const char*, const char*);

typedef void (*OnStateEnteredOp)(const char*);

typedef void (*OnStateExitOp)(const char*);

typedef void (*OnStateCustomEventOp)(const char*);

typedef void (*OnStateErrorOp)(const char*);

typedef void (*OnStateMachineStartOp)(void);

typedef void (*OnStateMachineStopOp)(void);

typedef void (*OnStringInputValueChangeOp)(const char*, const char*, const char*);

typedef void (*OnNumericInputValueChangeOp)(const char*, float, float);

typedef void (*OnBooleanInputValueChangeOp)(const char*, bool, bool);

typedef void (*OnInputFiredOp)(const char*);

typedef struct StateMachineObserver {
  OnTransitionOp on_transition_op;
  OnStateEnteredOp on_state_entered_op;
  OnStateExitOp on_state_exit_op;
  OnStateCustomEventOp on_state_custom_event_op;
  OnStateErrorOp on_state_error_op;
  OnStateMachineStartOp on_state_machine_start_op;
  OnStateMachineStopOp on_state_machine_stop_op;
  OnStringInputValueChangeOp on_string_input_value_change_op;
  OnNumericInputValueChangeOp on_numeric_input_value_change_op;
  OnBooleanInputValueChangeOp on_boolean_input_value_change_op;
  OnInputFiredOp on_input_fired_op;
} StateMachineObserver;

typedef void (*OnOp)(void);

typedef void (*OnFrameOp)(float);

typedef void (*OnRenderOp)(float);

typedef void (*OnLoopOp)(uint32_t);

typedef struct Observer {
  OnOp on_load_op;
  OnOp on_load_error_op;
  OnOp on_play_op;
  OnOp on_pause_op;
  OnOp on_stop_op;
  OnFrameOp on_frame_op;
  OnRenderOp on_render_op;
  OnLoopOp on_loop_op;
  OnOp on_complete_op;
} Observer;

int32_t dotlottie_active_animation_id(struct DotLottiePlayer *ptr, char *result);

int32_t dotlottie_active_theme_id(struct DotLottiePlayer *ptr, char *result);

int32_t dotlottie_animation_size(struct DotLottiePlayer *ptr, float *width, float *height);

int32_t dotlottie_buffer_len(struct DotLottiePlayer *ptr, uint64_t *result);

int32_t dotlottie_buffer_ptr(struct DotLottiePlayer *ptr, const uint32_t **result);

int32_t dotlottie_clear(struct DotLottiePlayer *ptr);

int32_t dotlottie_config(struct DotLottiePlayer *ptr, struct DotLottieConfig *result);

int32_t dotlottie_current_frame(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_destroy(struct DotLottiePlayer *ptr);

int32_t dotlottie_duration(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_init_config(struct DotLottieConfig *config);

int32_t dotlottie_is_complete(struct DotLottiePlayer *ptr, bool *result);

int32_t dotlottie_is_loaded(struct DotLottiePlayer *ptr);

int32_t dotlottie_is_paused(struct DotLottiePlayer *ptr);

int32_t dotlottie_is_playing(struct DotLottiePlayer *ptr);

int32_t dotlottie_is_stopped(struct DotLottiePlayer *ptr);

int32_t dotlottie_layer_bounds(struct DotLottiePlayer *ptr,
                               const char *layer_name,
                               struct LayerBoundingBox *bounding_box);

int32_t dotlottie_load_animation(struct DotLottiePlayer *ptr,
                                 const char *animation_id,
                                 uint32_t width,
                                 uint32_t height);

int32_t dotlottie_load_animation_data(struct DotLottiePlayer *ptr,
                                      const char *animation_data,
                                      uint32_t width,
                                      uint32_t height);

int32_t dotlottie_load_animation_path(struct DotLottiePlayer *ptr,
                                      const char *animation_path,
                                      uint32_t width,
                                      uint32_t height);

int32_t dotlottie_load_dotlottie_data(struct DotLottiePlayer *ptr,
                                      const char *file_data,
                                      size_t file_size,
                                      uint32_t width,
                                      uint32_t height);

int32_t dotlottie_loop_count(struct DotLottiePlayer *ptr, uint32_t *result);

int32_t dotlottie_manifest(struct DotLottiePlayer *ptr, struct DotLottieManifest *result);

int32_t dotlottie_manifest_animations(struct DotLottiePlayer *ptr,
                                      struct DotLottieManifestAnimation *result,
                                      size_t *size);

int32_t dotlottie_manifest_state_machines(struct DotLottiePlayer *ptr,
                                          struct DotLottieManifestStateMachine *result,
                                          size_t *size);

int32_t dotlottie_manifest_themes(struct DotLottiePlayer *ptr,
                                  struct DotLottieManifestTheme *result,
                                  size_t *size);

int32_t dotlottie_markers(struct DotLottiePlayer *ptr,
                          struct DotLottieMarker *result,
                          size_t *size);

struct DotLottiePlayer *dotlottie_new_player(const struct DotLottieConfig *ptr);

int32_t dotlottie_pause(struct DotLottiePlayer *ptr);

int32_t dotlottie_play(struct DotLottiePlayer *ptr);

int32_t dotlottie_register_font(const char *font_name,
                                const char *font_data,
                                size_t font_data_size);

int32_t dotlottie_render(struct DotLottiePlayer *ptr);

int32_t dotlottie_request_frame(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_reset_theme(struct DotLottiePlayer *ptr);

int32_t dotlottie_resize(struct DotLottiePlayer *ptr, uint32_t width, uint32_t height);

int32_t dotlottie_seek(struct DotLottiePlayer *ptr, float no);

int32_t dotlottie_segment_duration(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_set_frame(struct DotLottiePlayer *ptr, float no);

int32_t dotlottie_set_theme(struct DotLottiePlayer *ptr, const char *theme_id);

int32_t dotlottie_set_theme_data(struct DotLottiePlayer *ptr, const char *theme_data);

int32_t dotlottie_set_viewport(struct DotLottiePlayer *ptr,
                               int32_t x,
                               int32_t y,
                               int32_t w,
                               int32_t h);

int32_t dotlottie_state_machine_current_state(struct DotLottiePlayer *ptr, char *result);

int32_t dotlottie_state_machine_framework_setup(struct DotLottiePlayer *ptr, uint16_t *result);

int32_t dotlottie_state_machine_internal_subscribe(struct DotLottiePlayer *ptr,
                                                   struct StateMachineInternalObserver *observer);

int32_t dotlottie_state_machine_internal_unsubscribe(struct DotLottiePlayer *ptr,
                                                     struct StateMachineInternalObserver *observer);

int32_t dotlottie_state_machine_load(struct DotLottiePlayer *ptr, const char *state_machine_id);

int32_t dotlottie_state_machine_load_data(struct DotLottiePlayer *ptr,
                                          const char *state_machine_definition);

int32_t dotlottie_state_machine_override_current_state(struct DotLottiePlayer *ptr,
                                                       const char *state_name,
                                                       bool do_tick);

int32_t dotlottie_state_machine_post_event(struct DotLottiePlayer *ptr,
                                           const struct DotLottieEvent *event);

int32_t dotlottie_state_machine_set_boolean_input(struct DotLottiePlayer *ptr,
                                                  const char *key,
                                                  bool value);

int32_t dotlottie_state_machine_set_numeric_input(struct DotLottiePlayer *ptr,
                                                  const char *key,
                                                  float value);

int32_t dotlottie_state_machine_set_string_input(struct DotLottiePlayer *ptr,
                                                 const char *key,
                                                 const char *value);

int32_t dotlottie_state_machine_status(struct DotLottiePlayer *ptr, char *result);

int32_t dotlottie_state_machine_stop(struct DotLottiePlayer *ptr);

int32_t dotlottie_state_machine_subscribe(struct DotLottiePlayer *ptr,
                                          struct StateMachineObserver *observer);

int32_t dotlottie_state_machine_unsubscribe(struct DotLottiePlayer *ptr,
                                            struct StateMachineObserver *observer);

int32_t dotlottie_stop(struct DotLottiePlayer *ptr);

int32_t dotlottie_subscribe(struct DotLottiePlayer *ptr, struct Observer *observer);

int32_t dotlottie_total_frames(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_unsubscribe(struct DotLottiePlayer *ptr, struct Observer *observer);
