#include "../bindings.h"
#include <stdint.h>

int main() {
  float f = 1;
  uint8_t c = 'a';
  DotLottieConfig config = {
      .mode = Forward,
      .loop_animation = false,
      .speed = 0,
      .use_frame_interpolation = true,
      .autoplay = true,
      .segment = {.ptr = &f, .size = 1},
      .background_color = 1,
      .layout = {.fit = Contain, .align = {.ptr = &f, .size = 1}},
      .marker = {.ptr = &c, .size = 1}};
  DotLottiePlayer *ptr = new_dotlottie_player(&config);
}
