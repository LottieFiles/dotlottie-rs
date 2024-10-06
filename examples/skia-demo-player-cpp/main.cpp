#include <SDL.h>
#include <SDL_image.h>
#include <SDL_pixels.h>
#include <libgen.h> // For dirname
#include <limits.h> // For PATH_MAX
#include <memory>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h> // For readlink

extern "C" {
#include "../../dotlottie-ffi/bindings.h"
}

#include "include/codec/SkCodec.h"
#include "include/core/SkAlphaType.h"
#include "include/core/SkBitmap.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkColorType.h"
#include "include/core/SkData.h"
#include "include/core/SkGraphics.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkSurface.h"

#define WIDTH 1000
#define HEIGHT 1000

void usage(char *app) {
  fprintf(stderr, "usage: %s <animation-file>\n", app);
  exit(1);
}

int main(int argc, char **argv) {
  SDL_Window *window = NULL;
  SDL_Renderer *renderer = NULL;
  SDL_Texture *texture = NULL;
  SDL_Event e;

  DotLottieConfig config;
  DotLottiePlayer *player;

  const char *animation_path;
  int screen;
  const uint32_t *buffer;
  int len;
  char key_pressed[255];
  int ret;
  int ready;
  float current_frame;
  float next_frame;

  // Ensure a file path has been provided
  if (argc != 2) {
    usage(argv[0]);
  }
  // Ensure the file path is readable
  animation_path = argv[1];
  ret = access(animation_path, R_OK);
  if (ret != 0) {
    fprintf(stderr, "Invalid animation path\n\n");
    usage(argv[0]);
  }

  // Setup dotlottie config
  dotlottie_init_config(&config);
  config.loop_animation = true;
  config.background_color = 0xffffffff;
  config.layout.fit = Void;
  config.layout.align_x = 1.0;
  config.layout.align_y = 0.5;
  strcpy(config.marker.value, "feather");

  // Setup dotlottie player
  player = dotlottie_new_player(&config);
  if (!player) {
    fprintf(stderr, "Could not create dotlottie player\n");
    return 1;
  }

  // Load the animation file
  ret = dotlottie_load_animation_path(player, animation_path, WIDTH, HEIGHT);
  if (ret != DOTLOTTIE_SUCCESS) {
    fprintf(stderr, "Could not load dotlottie animation file\n");
    return 1;
  }
  // Get direct access to the underlying buffer
  ret = dotlottie_buffer_ptr(player, &buffer);
  if (ret != DOTLOTTIE_SUCCESS) {
    fprintf(stderr, "Could not access underlying dotlottie buffer\n");
    return 1;
  }

  if (SDL_Init(SDL_INIT_VIDEO) < 0) {
    fprintf(stderr, "Could not initialize SDL: %s\n", SDL_GetError());
    return 1;
  }

  // Setup skia buffer
  SkImageInfo info = SkImageInfo::MakeN32Premul(WIDTH, HEIGHT);
  size_t rowBytes = info.minRowBytes();
  size_t size = info.computeByteSize(rowBytes);
  void *pixels = malloc(size);
  // Setup Skia canvas
  std::unique_ptr<SkCanvas> canvas = SkCanvas::MakeRasterDirect(info, pixels, rowBytes);

  // Setup SDL window
  window = SDL_CreateWindow("skia-demo-player-cpp", SDL_WINDOWPOS_UNDEFINED,
                            SDL_WINDOWPOS_UNDEFINED, WIDTH, HEIGHT, SDL_WINDOW_SHOWN);
  if (!window) {
    fprintf(stderr, "Could not create SDL window: %s\n", SDL_GetError());
    ret = 1;
    goto quit;
  }
  renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
  if (!renderer) {
    fprintf(stderr, "Could not create SDL renderer: %s\n", SDL_GetError());
    ret = 1;
    goto quit;
  }
  texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_BGRA32, SDL_TEXTUREACCESS_STREAMING, WIDTH,
                              HEIGHT);
  if (!texture) {
    fprintf(stderr, "Could not create SDL texture: %s\n", SDL_GetError());
    ret = 1;
    goto quit;
  }
  SDL_UpdateTexture(texture, NULL, pixels, WIDTH * sizeof(Uint32));

  current_frame = 0;
  while (1) {
    // Process events
    while (SDL_PollEvent(&e) != 0) {
      if (e.type == SDL_QUIT) {
        goto quit;
      } else if (e.type == SDL_KEYDOWN) {
        switch (e.key.keysym.sym) {
        case SDLK_p:
          ret = dotlottie_play(player);
          if (ret != DOTLOTTIE_SUCCESS) {
            fprintf(stderr, "Could not start dotlottie player\n");
          }
          break;
        case SDLK_s:
          ret = dotlottie_stop(player);
          if (ret != DOTLOTTIE_SUCCESS) {
            fprintf(stderr, "Could not stop dotlottie player\n");
          }
          break;
        case SDLK_q:
          goto quit;
        }
      }
    }

    next_frame = 0;
    dotlottie_request_frame(player, &next_frame);
    if (next_frame != current_frame) {
      // Process the next frame
      dotlottie_set_frame(player, next_frame);
      dotlottie_render(player);
      // Use skia to render an image
      SkImageInfo imageInfo =
          SkImageInfo::Make(WIDTH, HEIGHT, kBGRA_8888_SkColorType, kPremul_SkAlphaType);
      sk_sp<SkData> imageData = SkData::MakeWithoutCopy(buffer, WIDTH * HEIGHT * 4);
      sk_sp<SkImage> bitmapImage = SkImages::RasterFromData(imageInfo, imageData, WIDTH * 4);
      // Draw the image
      SkRect src = SkRect::MakeWH(bitmapImage->width(), bitmapImage->height());
      SkRect dst = SkRect::MakeWH(WIDTH, HEIGHT);
      canvas->drawImageRect(bitmapImage, src, dst, SkSamplingOptions(), nullptr,
                            SkCanvas::kStrict_SrcRectConstraint);
      // Render the image in the window
      SDL_UpdateTexture(texture, NULL, pixels, WIDTH * sizeof(Uint32));
      SDL_RenderCopy(renderer, texture, NULL, NULL);
      SDL_RenderPresent(renderer);
      current_frame = next_frame;
    }
  }

quit:
  // Clean up
  if (texture)
    SDL_DestroyTexture(texture);
  if (renderer)
    SDL_DestroyRenderer(renderer);
  if (window)
    SDL_DestroyWindow(window);
  SDL_Quit();

  return ret;
}
