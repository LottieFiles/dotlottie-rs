// C API demo: plays an animation in an SDL2 window.
//
//   make native                       (from the repository root)
//   make -C dotlottie-rs/examples/c_api run ANIMATION_PATH=path/to/animation.lottie   (or .json)
//
//   P  play      S  stop      Q  quit

#include <SDL.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "dotlottie_player.h"

#define WIDTH 1000
#define HEIGHT 1000

static bool load_animation(DotLottiePlayer *player, const char *path) {
  const char *extension = strrchr(path, '.');
  if (!extension || strcmp(extension, ".lottie") != 0) {
    return dotlottie_load_animation_path(player, path) == DOTLOTTIE_SUCCESS;
  }

  FILE *file = fopen(path, "rb");
  if (!file) {
    return false;
  }
  fseek(file, 0, SEEK_END);
  long size = ftell(file);
  rewind(file);

  bool loaded = false;
  char *data = size > 0 ? malloc((size_t)size) : NULL;
  if (data && fread(data, 1, (size_t)size, file) == (size_t)size) {
    loaded = dotlottie_load_dotlottie_data(player, data, (size_t)size) == DOTLOTTIE_SUCCESS;
  }

  free(data);
  fclose(file);
  return loaded;
}

int main(int argc, char **argv) {
  SDL_Window *window = NULL;
  SDL_Renderer *renderer = NULL;
  SDL_Texture *texture = NULL;
  DotLottiePlayer *player = NULL;
  uint32_t *buffer = NULL;
  int status = 1;

  if (argc != 2 || access(argv[1], R_OK) != 0) {
    fprintf(stderr, "usage: %s <animation-file>\n", argv[0]);
    return 1;
  }

  player = dotlottie_new_player(0);
  if (!player) {
    fprintf(stderr, "Could not create dotlottie player\n");
    return 1;
  }

  buffer = malloc(WIDTH * HEIGHT * sizeof(uint32_t));
  if (!buffer) {
    fprintf(stderr, "Could not allocate buffer\n");
    goto quit;
  }

  if (dotlottie_set_sw_target(player, buffer, WIDTH, WIDTH, HEIGHT, ABGR8888) != DOTLOTTIE_SUCCESS) {
    fprintf(stderr, "Could not set software rendering target\n");
    goto quit;
  }

  dotlottie_set_loop(player, true);
  dotlottie_set_autoplay(player, true);
  dotlottie_set_background(player, 255, 255, 255, 255);

  if (!load_animation(player, argv[1])) {
    fprintf(stderr, "Could not load animation file\n");
    goto quit;
  }

  if (SDL_Init(SDL_INIT_VIDEO) < 0) {
    fprintf(stderr, "Could not initialize SDL: %s\n", SDL_GetError());
    goto quit;
  }

  window = SDL_CreateWindow("dotLottie C API demo", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
                            WIDTH, HEIGHT, SDL_WINDOW_SHOWN);
  if (!window) {
    fprintf(stderr, "Could not create SDL window: %s\n", SDL_GetError());
    goto quit;
  }
  renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC);
  if (!renderer) {
    fprintf(stderr, "Could not create SDL renderer: %s\n", SDL_GetError());
    goto quit;
  }
  texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ABGR8888, SDL_TEXTUREACCESS_STREAMING,
                              WIDTH, HEIGHT);
  if (!texture) {
    fprintf(stderr, "Could not create SDL texture: %s\n", SDL_GetError());
    goto quit;
  }

  status = 0;
  Uint64 last = SDL_GetTicks64();

  while (1) {
    SDL_Event event;
    while (SDL_PollEvent(&event) != 0) {
      if (event.type == SDL_QUIT) {
        goto quit;
      }
      if (event.type == SDL_KEYDOWN) {
        switch (event.key.keysym.sym) {
        case SDLK_p:
          dotlottie_play(player);
          break;
        case SDLK_s:
          dotlottie_stop(player);
          break;
        case SDLK_q:
          goto quit;
        }
      }
    }

    Uint64 now = SDL_GetTicks64();
    float dt = (float)(now - last);
    last = now;

    bool rendered = false;
    dotlottie_tick(player, dt, &rendered);
    if (rendered) {
      SDL_UpdateTexture(texture, NULL, buffer, WIDTH * sizeof(uint32_t));
    }

    SDL_RenderCopy(renderer, texture, NULL, NULL);
    SDL_RenderPresent(renderer);
  }

quit:
  // The player holds a pointer into the buffer, so destroy it first.
  if (player)
    dotlottie_destroy(player);
  free(buffer);
  if (texture)
    SDL_DestroyTexture(texture);
  if (renderer)
    SDL_DestroyRenderer(renderer);
  if (window)
    SDL_DestroyWindow(window);
  SDL_Quit();

  return status;
}
