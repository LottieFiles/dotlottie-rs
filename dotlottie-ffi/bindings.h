#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


#define DOTLOTTIE_ERROR 1

#define DOTLOTTIE_INVALID_PARAMETER 2

#define DOTLOTTIE_MANIFEST_NOT_AVAILABLE 3

#define DOTLOTTIE_MANIFEST_THEMES_NOT_AVAILABLE 4

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
  float speed;
  bool use_frame_interpolation;
  bool autoplay;
  float segment_start;
  float segment_end;
  uint32_t background_color;
  struct DotLottieLayout layout;
  struct DotLottieString marker;
} DotLottieConfig;

typedef struct LayerBoundingBox {
  float x;
  float y;
  float w;
  float h;
} LayerBoundingBox;

typedef struct DotLottieOption_DotLottieString {
  struct DotLottieString value;
  bool defined;
} DotLottieOption_DotLottieString;

typedef struct DotLottieOption_u32 {
  uint32_t value;
  bool defined;
} DotLottieOption_u32;

typedef struct DotLottieManifest {
  struct DotLottieOption_DotLottieString active_animation_id;
  struct DotLottieOption_DotLottieString author;
  struct DotLottieOption_DotLottieString description;
  struct DotLottieOption_DotLottieString generator;
  struct DotLottieOption_DotLottieString keywords;
  struct DotLottieOption_u32 revision;
  struct DotLottieOption_DotLottieString version;
} DotLottieManifest;

typedef struct DotLottieOption_bool {
  bool value;
  bool defined;
} DotLottieOption_bool;

typedef struct DotLottieOption_i8 {
  int8_t value;
  bool defined;
} DotLottieOption_i8;

typedef struct DotLottieOption_f32 {
  float value;
  bool defined;
} DotLottieOption_f32;

typedef struct DotLottieManifestAnimation {
  struct DotLottieOption_bool autoplay;
  struct DotLottieOption_DotLottieString default_theme;
  struct DotLottieOption_i8 direction;
  struct DotLottieOption_bool hover;
  struct DotLottieOption_DotLottieString id;
  struct DotLottieOption_u32 intermission;
  struct DotLottieOption_bool loop;
  struct DotLottieOption_u32 loop_count;
  struct DotLottieOption_DotLottieString play_mode;
  struct DotLottieOption_f32 speed;
  struct DotLottieOption_DotLottieString theme_color;
} DotLottieManifestAnimation;

typedef struct DotLottieManifestState {
  struct DotLottieString state;
} DotLottieManifestState;

typedef struct DotLottieManifestTheme {
  struct DotLottieString id;
} DotLottieManifestTheme;

typedef struct DotLottieManifestThemeAnimation {
  struct DotLottieString id;
} DotLottieManifestThemeAnimation;

typedef struct DotLottieMarker {
  struct DotLottieString name;
  float duration;
  float time;
} DotLottieMarker;

typedef enum DotLottieEvent_Tag {
  Boolean,
  String,
  Numeric,
  OnPointerDown,
  OnPointerUp,
  OnPointerMove,
  OnPointerEnter,
  OnPointerExit,
  OnComplete,
  SetNumericContext,
} DotLottieEvent_Tag;

typedef struct Boolean_Body {
  bool value;
} Boolean_Body;

typedef struct String_Body {
  struct DotLottieString value;
} String_Body;

typedef struct Numeric_Body {
  float value;
} Numeric_Body;

typedef struct OnPointerDown_Body {
  float x;
  float y;
} OnPointerDown_Body;

typedef struct OnPointerUp_Body {
  float x;
  float y;
} OnPointerUp_Body;

typedef struct OnPointerMove_Body {
  float x;
  float y;
} OnPointerMove_Body;

typedef struct OnPointerEnter_Body {
  float x;
  float y;
} OnPointerEnter_Body;

typedef struct OnPointerExit_Body {
  float x;
  float y;
} OnPointerExit_Body;

typedef struct SetNumericContext_Body {
  struct DotLottieString key;
  float value;
} SetNumericContext_Body;

typedef struct DotLottieEvent {
  DotLottieEvent_Tag tag;
  union {
    Boolean_Body boolean;
    String_Body string;
    Numeric_Body numeric;
    OnPointerDown_Body on_pointer_down;
    OnPointerUp_Body on_pointer_up;
    OnPointerMove_Body on_pointer_move;
    OnPointerEnter_Body on_pointer_enter;
    OnPointerExit_Body on_pointer_exit;
    SetNumericContext_Body set_numeric_context;
  };
} DotLottieEvent;

typedef void (*OnTransitionOp)(const char*, const char*);

typedef void (*OnStateEnteredOp)(const char*);

typedef void (*OnStateExitOp)(const char*);

typedef struct StateMachineObserver {
  OnTransitionOp on_transition_op;
  OnStateEnteredOp on_state_entered_op;
  OnStateExitOp on_state_exit_op;
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

int32_t dotlottie_load_state_machine(struct DotLottiePlayer *ptr, const char *state_machine_id);

int32_t dotlottie_load_state_machine_data(struct DotLottiePlayer *ptr,
                                          const char *state_machine_definition);

int32_t dotlottie_load_theme(struct DotLottiePlayer *ptr, const char *theme_id);

int32_t dotlottie_load_theme_data(struct DotLottiePlayer *ptr, const char *theme_data);

int32_t dotlottie_loop_count(struct DotLottiePlayer *ptr, uint32_t *result);

int32_t dotlottie_manifest(struct DotLottiePlayer *ptr, struct DotLottieManifest *result);

int32_t dotlottie_manifest_animations(struct DotLottiePlayer *ptr,
                                      struct DotLottieManifestAnimation *result,
                                      size_t *size);

int32_t dotlottie_manifest_states(struct DotLottiePlayer *ptr,
                                  struct DotLottieManifestState *result,
                                  size_t *size);

int32_t dotlottie_manifest_theme_animations(struct DotLottiePlayer *ptr,
                                            const struct DotLottieManifestTheme *theme,
                                            struct DotLottieManifestThemeAnimation *result,
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

int32_t dotlottie_post_event(struct DotLottiePlayer *ptr, const struct DotLottieEvent *event);

int32_t dotlottie_render(struct DotLottiePlayer *ptr);

int32_t dotlottie_request_frame(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_resize(struct DotLottiePlayer *ptr, uint32_t width, uint32_t height);

int32_t dotlottie_seek(struct DotLottiePlayer *ptr, float no);

int32_t dotlottie_segment_duration(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_set_frame(struct DotLottiePlayer *ptr, float no);

int32_t dotlottie_set_state_machine_boolean_context(struct DotLottiePlayer *ptr,
                                                    const char *key,
                                                    bool value);

int32_t dotlottie_set_state_machine_numeric_context(struct DotLottiePlayer *ptr,
                                                    const char *key,
                                                    float value);

int32_t dotlottie_set_state_machine_string_context(struct DotLottiePlayer *ptr,
                                                   const char *key,
                                                   const char *value);

int32_t dotlottie_set_viewport(struct DotLottiePlayer *ptr,
                               int32_t x,
                               int32_t y,
                               int32_t w,
                               int32_t h);

int32_t dotlottie_start_state_machine(struct DotLottiePlayer *ptr);

int32_t dotlottie_state_machine_framework_setup(struct DotLottiePlayer *ptr, uint16_t *result);

int32_t dotlottie_state_machine_subscribe(struct DotLottiePlayer *ptr,
                                          struct StateMachineObserver *observer);

int32_t dotlottie_state_machine_unsubscribe(struct DotLottiePlayer *ptr,
                                            struct StateMachineObserver *observer);

int32_t dotlottie_stop(struct DotLottiePlayer *ptr);

int32_t dotlottie_stop_state_machine(struct DotLottiePlayer *ptr);

int32_t dotlottie_subscribe(struct DotLottiePlayer *ptr, struct Observer *observer);

int32_t dotlottie_total_frames(struct DotLottiePlayer *ptr, float *result);

int32_t dotlottie_unsubscribe(struct DotLottiePlayer *ptr, struct Observer *observer);
