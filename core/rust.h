#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct DotLottiePlayer {
  bool autoplay;
  bool loop_animation;
  int32_t speed;
  int8_t direction;
  float duration;
  uint32_t current_frame;
  uint32_t total_frames;
  Tvg_Animation *animation;
  Tvg_Canvas *canvas;
} DotLottiePlayer;

struct DotLottiePlayer *create_dotlottie_player(bool autoplay,
                                                bool loop_animation,
                                                int8_t direction,
                                                int32_t speed);

void tick(struct DotLottiePlayer *ptr);

void load_animation(struct DotLottiePlayer *ptr,
                    uint32_t *buffer,
                    const char *animation_data,
                    uint32_t width,
                    uint32_t height);

void destroy_dotlottie_player(struct DotLottiePlayer *ptr);
