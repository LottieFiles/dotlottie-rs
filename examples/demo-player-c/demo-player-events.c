#include <SDL.h>
#include <SDL_ttf.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "dotlottie_player.h"

#define WIDTH 800
#define HEIGHT 600
#define ANIMATION_SIZE 500
#define BUTTON_WIDTH 120
#define BUTTON_HEIGHT 40
#define BUTTON_SPACING 10
#define UI_PADDING 20

typedef struct {
  SDL_Rect rect;
  const char *label;
  bool is_hovered;
} Button;

typedef struct {
  Button play_pause;
  Button reset;
} UIButtons;

void draw_button(SDL_Renderer *renderer, TTF_Font *font, Button *button,
                 bool is_active) {
  // Button background
  SDL_Color bg_color;
  if (button->is_hovered) {
    bg_color = (SDL_Color){70, 130, 180, 255}; // Steel blue when hovered
  } else if (is_active) {
    bg_color = (SDL_Color){50, 150, 50, 255}; // Green when active
  } else {
    bg_color = (SDL_Color){60, 60, 60, 255}; // Dark gray
  }

  SDL_SetRenderDrawColor(renderer, bg_color.r, bg_color.g, bg_color.b,
                         bg_color.a);
  SDL_RenderFillRect(renderer, &button->rect);

  // Button border
  SDL_SetRenderDrawColor(renderer, 200, 200, 200, 255);
  SDL_RenderDrawRect(renderer, &button->rect);

  // Button text
  SDL_Color text_color = {255, 255, 255, 255};
  SDL_Surface *text_surface =
      TTF_RenderText_Blended(font, button->label, text_color);
  if (text_surface) {
    SDL_Texture *text_texture =
        SDL_CreateTextureFromSurface(renderer, text_surface);
    if (text_texture) {
      SDL_Rect text_rect = {
          button->rect.x + (button->rect.w - text_surface->w) / 2,
          button->rect.y + (button->rect.h - text_surface->h) / 2,
          text_surface->w, text_surface->h};
      SDL_RenderCopy(renderer, text_texture, NULL, &text_rect);
      SDL_DestroyTexture(text_texture);
    }
    SDL_FreeSurface(text_surface);
  }
}

void draw_event_log(SDL_Renderer *renderer, TTF_Font *font,
                    const char *event_text) {
  SDL_Color text_color = {255, 255, 255, 255};
  SDL_Surface *text_surface =
      TTF_RenderText_Blended(font, event_text, text_color);
  if (text_surface) {
    SDL_Texture *text_texture =
        SDL_CreateTextureFromSurface(renderer, text_surface);
    if (text_texture) {
      SDL_Rect text_rect = {UI_PADDING, HEIGHT - 60, text_surface->w,
                            text_surface->h};
      SDL_RenderCopy(renderer, text_texture, NULL, &text_rect);
      SDL_DestroyTexture(text_texture);
    }
    SDL_FreeSurface(text_surface);
  }
}

bool is_point_in_button(int x, int y, Button *button) {
  return x >= button->rect.x && x <= button->rect.x + button->rect.w &&
         y >= button->rect.y && y <= button->rect.y + button->rect.h;
}

void usage(char *app) {
  fprintf(stderr, "usage: %s <animation-file>\n", app);
  exit(1);
}

int main(int argc, char **argv) {
  SDL_Window *window = NULL;
  SDL_Renderer *renderer = NULL;
  SDL_Texture *texture = NULL;
  SDL_Event e;
  TTF_Font *font = NULL;

  DotLottieConfig config;
  DotLottiePlayer *player;

  const char *animation_path;
  const uint32_t *buffer;
  int ret;
  bool is_playing = false;
  char event_log[256] = "Events: None";
  char event_buffer[256];

  UIButtons buttons;

  // Ensure a file path has been provided
  if (argc != 2) {
    usage(argv[0]);
  }

  animation_path = argv[1];
  ret = access(animation_path, R_OK);
  if (ret != 0) {
    fprintf(stderr, "Invalid animation path\n\n");
    usage(argv[0]);
  }

  // Initialize SDL
  if (SDL_Init(SDL_INIT_VIDEO) < 0) {
    fprintf(stderr, "Could not initialize SDL: %s\n", SDL_GetError());
    return 1;
  }

  if (TTF_Init() < 0) {
    fprintf(stderr, "Could not initialize SDL_ttf: %s\n", TTF_GetError());
    SDL_Quit();
    return 1;
  }

  // Load font
  font = TTF_OpenFont("/System/Library/Fonts/Helvetica.ttc", 16);
  if (!font) {
    // Try alternative font path
    font = TTF_OpenFont("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 16);
    if (!font) {
      fprintf(stderr, "Warning: Could not load font: %s\n", TTF_GetError());
    }
  }

  // Setup dotlottie config
  dotlottie_init_config(&config);
  config.loop_animation = true; // Enable looping by default
  // config.loop_count = 2u;
  config.background_color = 0xff1a1a1a;
  config.layout.fit = Contain;
  config.layout.align_x = 0.5;
  config.layout.align_y = 0.5;
  config.autoplay = false;

  // Setup dotlottie player
  player = dotlottie_new_player(&config);
  if (!player) {
    fprintf(stderr, "Could not create dotlottie player\n");
    ret = 1;
    goto quit;
  }

  // Load the animation file
  ret = dotlottie_load_animation_path(player, animation_path, ANIMATION_SIZE,
                                      ANIMATION_SIZE);
  if (ret != DOTLOTTIE_SUCCESS) {
    fprintf(stderr, "Could not load dotlottie animation file\n");
    ret = 1;
    goto quit;
  }

  // Get direct access to the underlying buffer
  ret = dotlottie_buffer_ptr(player, &buffer);
  if (ret != DOTLOTTIE_SUCCESS) {
    fprintf(stderr, "Could not access underlying dotlottie buffer\n");
    ret = 1;
    goto quit;
  }

  // Setup SDL window
  window = SDL_CreateWindow(
      "DotLottie Event Polling Demo", SDL_WINDOWPOS_UNDEFINED,
      SDL_WINDOWPOS_UNDEFINED, WIDTH, HEIGHT,
      SDL_WINDOW_SHOWN | SDL_WINDOW_ALWAYS_ON_TOP | SDL_WINDOW_INPUT_FOCUS);

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

  texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_BGRA32,
                              SDL_TEXTUREACCESS_STREAMING, ANIMATION_SIZE,
                              ANIMATION_SIZE);

  if (!texture) {
    fprintf(stderr, "Could not create SDL texture: %s\n", SDL_GetError());
    ret = 1;
    goto quit;
  }

  // Setup UI buttons
  int button_y = HEIGHT - 120;
  int button_x = UI_PADDING;

  buttons.play_pause =
      (Button){.rect = {button_x, button_y, BUTTON_WIDTH, BUTTON_HEIGHT},
               .label = "Play",
               .is_hovered = false};

  button_x += BUTTON_WIDTH + BUTTON_SPACING;
  buttons.reset =
      (Button){.rect = {button_x, button_y, BUTTON_WIDTH, BUTTON_HEIGHT},
               .label = "Reset",
               .is_hovered = false};

  printf("DotLottie Event Polling Demo\n");
  printf("Controls:\n");
  printf("  - Click Play/Pause button to control playback\n");
  printf("  - Click Reset to restart animation\n");
  printf("  - Press Q or ESC to quit\n");
  printf("\nWatching for events...\n\n");

  // Main loop
  bool quit_app = false;
  Uint32 last_tick = SDL_GetTicks();

  while (!quit_app) {
    // Process SDL events
    while (SDL_PollEvent(&e)) {
      if (e.type == SDL_QUIT) {
        quit_app = true;
      } else if (e.type == SDL_KEYDOWN) {
        if (e.key.keysym.sym == SDLK_q || e.key.keysym.sym == SDLK_ESCAPE) {
          quit_app = true;
        }
      } else if (e.type == SDL_MOUSEMOTION) {
        int mx = e.motion.x;
        int my = e.motion.y;
        buttons.play_pause.is_hovered =
            is_point_in_button(mx, my, &buttons.play_pause);
        buttons.reset.is_hovered = is_point_in_button(mx, my, &buttons.reset);
      } else if (e.type == SDL_MOUSEBUTTONDOWN) {
        int mx = e.button.x;
        int my = e.button.y;

        if (is_point_in_button(mx, my, &buttons.play_pause)) {
          if (is_playing) {
            dotlottie_pause(player);
            buttons.play_pause.label = "Play";
          } else {
            dotlottie_play(player);
            buttons.play_pause.label = "Pause";
          }
          is_playing = !is_playing;
        } else if (is_point_in_button(mx, my, &buttons.reset)) {
          dotlottie_stop(player);
          dotlottie_play(player);
          is_playing = true;
          buttons.play_pause.label = "Pause";
        }
      }
    }

    // Poll DotLottie events
    struct dotlottieDotLottiePlayerEvent player_event;
    while (dotlottie_poll_event(player, &player_event) == 1) {
      switch (player_event.event_type) {
      case Load:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Load");
        printf("✓ Load event\n");
        break;
      case LoadError:
        snprintf(event_buffer, sizeof(event_buffer), "Event: LoadError");
        printf("✗ LoadError event\n");
        break;
      case Play:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Play");
        printf("▶ Play event\n");
        break;
      case Pause:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Pause");
        printf("⏸ Pause event\n");
        break;
      case Stop:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Stop");
        printf("⏹ Stop event\n");
        break;
      case Frame:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Frame %.1f",
                 player_event.data.frame_no);
        // printf("🔄 Frame event (Frame: %f)\n", player_event.data.frame_no);
        break;
      case Render:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Render %.1f",
                 player_event.data.frame_no);
        // printf("🔄 Render event (Frame: %f)\n", player_event.data.frame_no);
        break;
      case Loop:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Loop (count: %u)",
                 player_event.data.loop_count);
        printf("🔄 Loop event (count: %u)\n", player_event.data.loop_count);
        break;
      case Complete:
        snprintf(event_buffer, sizeof(event_buffer), "Event: Complete");
        printf("✓ Complete event\n");
        break;
      }

      strncpy(event_log, event_buffer, sizeof(event_log) - 1);
    }

    // Tick the player
    Uint32 current_tick = SDL_GetTicks();
    if (current_tick - last_tick >= 16) { // ~60 FPS
      dotlottie_tick(player);
      last_tick = current_tick;
    }

    // Clear screen
    SDL_SetRenderDrawColor(renderer, 26, 26, 26, 255);
    SDL_RenderClear(renderer);

    // Draw animation
    SDL_UpdateTexture(texture, NULL, buffer, ANIMATION_SIZE * sizeof(Uint32));
    SDL_Rect anim_rect = {(WIDTH - ANIMATION_SIZE) / 2, UI_PADDING,
                          ANIMATION_SIZE, ANIMATION_SIZE};
    SDL_RenderCopy(renderer, texture, NULL, &anim_rect);

    // Draw UI if font is available
    if (font) {
      draw_button(renderer, font, &buttons.play_pause, is_playing);
      draw_button(renderer, font, &buttons.reset, false);
      draw_event_log(renderer, font, event_log);
    }

    SDL_RenderPresent(renderer);

    // Small delay to prevent busy waiting
    SDL_Delay(1);
  }

  ret = 0;

quit:
  if (texture)
    SDL_DestroyTexture(texture);
  if (renderer)
    SDL_DestroyRenderer(renderer);
  if (window)
    SDL_DestroyWindow(window);
  if (font)
    TTF_CloseFont(font);
  TTF_Quit();
  SDL_Quit();

  return ret;
}
