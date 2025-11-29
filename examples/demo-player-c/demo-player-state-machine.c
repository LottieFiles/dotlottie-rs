#include <SDL.h>
#include <SDL_ttf.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>

#include "dotlottie_player.h"

#define WIDTH 900
#define HEIGHT 700
#define ANIMATION_SIZE 500
#define BUTTON_WIDTH 150
#define BUTTON_HEIGHT 40
#define BUTTON_SPACING 10
#define UI_PADDING 20
#define LOG_HEIGHT 200

typedef struct {
    SDL_Rect rect;
    const char *label;
    bool is_hovered;
} Button;

typedef struct {
    Button start_sm;
    Button stop_sm;
} UIButtons;

typedef struct {
    char lines[10][256];
    int count;
} EventLog;

void log_event(EventLog *log, const char *message) {
    if (log->count >= 10) {
        // Shift lines up
        for (int i = 0; i < 9; i++) {
            strncpy(log->lines[i], log->lines[i + 1], 255);
        }
        log->count = 9;
    }
    strncpy(log->lines[log->count], message, 255);
    log->lines[log->count][255] = '\0';
    log->count++;
}

void draw_button(SDL_Renderer *renderer, TTF_Font *font, Button *button, bool is_active) {
    SDL_Color bg_color;
    if (button->is_hovered) {
        bg_color = (SDL_Color){70, 130, 180, 255};
    } else if (is_active) {
        bg_color = (SDL_Color){50, 150, 50, 255};
    } else {
        bg_color = (SDL_Color){60, 60, 60, 255};
    }

    SDL_SetRenderDrawColor(renderer, bg_color.r, bg_color.g, bg_color.b, bg_color.a);
    SDL_RenderFillRect(renderer, &button->rect);

    SDL_SetRenderDrawColor(renderer, 200, 200, 200, 255);
    SDL_RenderDrawRect(renderer, &button->rect);

    if (font) {
        SDL_Color text_color = {255, 255, 255, 255};
        SDL_Surface *text_surface = TTF_RenderText_Blended(font, button->label, text_color);
        if (text_surface) {
            SDL_Texture *text_texture = SDL_CreateTextureFromSurface(renderer, text_surface);
            if (text_texture) {
                SDL_Rect text_rect = {
                    button->rect.x + (button->rect.w - text_surface->w) / 2,
                    button->rect.y + (button->rect.h - text_surface->h) / 2,
                    text_surface->w,
                    text_surface->h
                };
                SDL_RenderCopy(renderer, text_texture, NULL, &text_rect);
                SDL_DestroyTexture(text_texture);
            }
            SDL_FreeSurface(text_surface);
        }
    }
}

void draw_event_log(SDL_Renderer *renderer, TTF_Font *font, EventLog *log, int x, int y) {
    if (!font) return;

    SDL_Color text_color = {255, 255, 255, 255};
    int line_height = 18;

    for (int i = 0; i < log->count; i++) {
        SDL_Surface *text_surface = TTF_RenderText_Blended(font, log->lines[i], text_color);
        if (text_surface) {
            SDL_Texture *text_texture = SDL_CreateTextureFromSurface(renderer, text_surface);
            if (text_texture) {
                SDL_Rect text_rect = {x, y + i * line_height, text_surface->w, text_surface->h};
                SDL_RenderCopy(renderer, text_texture, NULL, &text_rect);
                SDL_DestroyTexture(text_texture);
            }
            SDL_FreeSurface(text_surface);
        }
    }
}

bool is_point_in_button(int x, int y, Button *button) {
    return x >= button->rect.x && x <= button->rect.x + button->rect.w &&
           y >= button->rect.y && y <= button->rect.y + button->rect.h;
}

bool is_point_in_rect(int x, int y, SDL_Rect *rect) {
    return x >= rect->x && x <= rect->x + rect->w &&
           y >= rect->y && y <= rect->y + rect->h;
}

void usage(char *app) {
    fprintf(stderr, "usage: %s <dotlottie-file-with-state-machine>\n", app);
    fprintf(stderr, "\nExample .lottie files with state machines:\n");
    fprintf(stderr, "  - Files with .lottie extension that contain state machine definitions\n");
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
    struct dotlottieStateMachineEngine *sm = NULL;

    const char *animation_path;
    const uint32_t *buffer;
    int ret;
    EventLog event_log = {.count = 0};
    char temp_msg[256];

    UIButtons buttons;
    SDL_Rect anim_rect;

    if (argc != 2) {
        usage(argv[0]);
    }

    animation_path = argv[1];
    ret = access(animation_path, R_OK);
    if (ret != 0) {
        fprintf(stderr, "Invalid animation path: %s\n\n", animation_path);
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

    font = TTF_OpenFont("/System/Library/Fonts/Helvetica.ttc", 14);
    if (!font) {
        font = TTF_OpenFont("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 14);
        if (!font) {
            fprintf(stderr, "Warning: Could not load font: %s\n", TTF_GetError());
        }
    }

    // Setup dotlottie config
    dotlottie_init_config(&config);
    config.loop_animation = true;
    config.background_color = 0xff1a1a1a;
    config.layout.fit = Contain;
    config.layout.align_x = 0.5;
    config.layout.align_y = 0.5;
    config.autoplay = true;

    player = dotlottie_new_player(&config);
    if (!player) {
        fprintf(stderr, "Could not create dotlottie player\n");
        ret = 1;
        goto quit;
    }

    // Check if file is a .lottie file (needs to be loaded as data)
    size_t path_len = strlen(animation_path);
    bool is_lottie = path_len > 7 && strcmp(animation_path + path_len - 7, ".lottie") == 0;

    if (is_lottie) {
        // Read file contents
        FILE *f = fopen(animation_path, "rb");
        if (!f) {
            fprintf(stderr, "Could not open file: %s\n", animation_path);
            ret = 1;
            goto quit;
        }

        fseek(f, 0, SEEK_END);
        long file_size = ftell(f);
        fseek(f, 0, SEEK_SET);

        char *file_data = malloc(file_size);
        if (!file_data) {
            fprintf(stderr, "Could not allocate memory for file\n");
            fclose(f);
            ret = 1;
            goto quit;
        }

        size_t bytes_read = fread(file_data, 1, file_size, f);
        fclose(f);

        if (bytes_read != file_size) {
            fprintf(stderr, "Could not read entire file\n");
            free(file_data);
            ret = 1;
            goto quit;
        }

        // Load as binary dotLottie data
        ret = dotlottie_load_dotlottie_data(player, file_data, file_size, ANIMATION_SIZE, ANIMATION_SIZE);
        free(file_data);
    } else {
        // Load as path (for .json files)
        ret = dotlottie_load_animation_path(player, animation_path, ANIMATION_SIZE, ANIMATION_SIZE);
    }

    if (ret != DOTLOTTIE_SUCCESS) {
        fprintf(stderr, "Could not load animation file: %s\n", animation_path);
        ret = 1;
        goto quit;
    }

    ret = dotlottie_buffer_ptr(player, &buffer);
    if (ret != DOTLOTTIE_SUCCESS) {
        fprintf(stderr, "Could not access underlying dotlottie buffer\n");
        ret = 1;
        goto quit;
    }

    window = SDL_CreateWindow(
        "DotLottie State Machine Demo",
        SDL_WINDOWPOS_UNDEFINED,
        SDL_WINDOWPOS_UNDEFINED,
        WIDTH,
        HEIGHT,
        SDL_WINDOW_SHOWN | SDL_WINDOW_ALWAYS_ON_TOP | SDL_WINDOW_INPUT_FOCUS
    );

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

    texture = SDL_CreateTexture(
        renderer,
        SDL_PIXELFORMAT_BGRA32,
        SDL_TEXTUREACCESS_STREAMING,
        ANIMATION_SIZE,
        ANIMATION_SIZE
    );

    if (!texture) {
        fprintf(stderr, "Could not create SDL texture: %s\n", SDL_GetError());
        ret = 1;
        goto quit;
    }

    // Setup animation rectangle
    anim_rect = (SDL_Rect){
        (WIDTH - ANIMATION_SIZE) / 2,
        UI_PADDING,
        ANIMATION_SIZE,
        ANIMATION_SIZE
    };

    // Setup UI buttons
    int button_y = ANIMATION_SIZE + UI_PADDING * 2;
    int button_x = (WIDTH - (BUTTON_WIDTH * 2 + BUTTON_SPACING)) / 2;

    buttons.start_sm = (Button){
        .rect = {button_x, button_y, BUTTON_WIDTH, BUTTON_HEIGHT},
        .label = "Start State Machine",
        .is_hovered = false
    };

    button_x += BUTTON_WIDTH + BUTTON_SPACING;
    buttons.stop_sm = (Button){
        .rect = {button_x, button_y, BUTTON_WIDTH, BUTTON_HEIGHT},
        .label = "Stop State Machine",
        .is_hovered = false
    };

    printf("=== DotLottie State Machine Demo ===\n");
    printf("Loaded: %s\n", animation_path);
    printf("\nControls:\n");
    printf("  - Click 'Start State Machine' to activate state machine\n");
    printf("  - Click 'Stop State Machine' to deactivate\n");
    printf("  - Click on the animation to send pointer events to state machine\n");
    printf("  - Press Q or ESC to quit\n");
    printf("\nState Machine Events:\n\n");

    log_event(&event_log, "Waiting for state machine...");

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
                buttons.start_sm.is_hovered = is_point_in_button(mx, my, &buttons.start_sm);
                buttons.stop_sm.is_hovered = is_point_in_button(mx, my, &buttons.stop_sm);
            } else if (e.type == SDL_MOUSEBUTTONDOWN) {
                int mx = e.button.x;
                int my = e.button.y;

                if (is_point_in_button(mx, my, &buttons.start_sm)) {
                    if (sm == NULL) {
                        // Load state machine (returns pointer, NULL on error)
                        sm = dotlottie_state_machine_load(player, "star-rating");
                        if (sm != NULL) {
                            // Start the state machine with default policy (NULL = use default)
                            ret = dotlottie_state_machine_start(sm, NULL);
                            if (ret == DOTLOTTIE_SUCCESS) {
                                log_event(&event_log, "▶ Started state machine");
                                printf("▶ State machine loaded and started\n");
                            } else {
                                log_event(&event_log, "❌ Failed to start SM");
                                printf("❌ State machine loaded but failed to start\n");
                                dotlottie_state_machine_release(sm);
                                sm = NULL;
                            }
                        } else {
                            log_event(&event_log, "❌ No state machine found");
                            printf("❌ Failed to load state machine\n");
                        }
                    }
                } else if (is_point_in_button(mx, my, &buttons.stop_sm)) {
                    if (sm != NULL) {
                        dotlottie_state_machine_stop(sm);
                        dotlottie_state_machine_release(sm);
                        sm = NULL;
                        log_event(&event_log, "⏹ Stopped state machine");
                        printf("⏹ Stopped and released state machine\n");
                    }
                } else if (is_point_in_rect(mx, my, &anim_rect)) {
                    // Click on animation area - send to state machine
                    if (sm != NULL) {
                        // Pixel coordinates relative to animation (0 to ANIMATION_SIZE)
                        float x = (float)(mx - anim_rect.x);
                        float y = (float)(my - anim_rect.y);

                        // Send pointer down event to state machine
                        dotlottie_state_machine_post_pointer_down(sm, x, y);

                        snprintf(temp_msg, sizeof(temp_msg), "🖱 Click at (%.0f, %.0f) px", x, y);
                        log_event(&event_log, temp_msg);
                        printf("🖱 Pointer down at (%.0f, %.0f) px\n", x, y);
                    }
                }
            } else if (e.type == SDL_MOUSEBUTTONUP) {
                int mx = e.button.x;
                int my = e.button.y;

                if (is_point_in_rect(mx, my, &anim_rect) && sm != NULL) {
                    // Pixel coordinates relative to animation
                    float x = (float)(mx - anim_rect.x);
                    float y = (float)(my - anim_rect.y);

                    // Send pointer up and click to state machine
                    dotlottie_state_machine_post_pointer_up(sm, x, y);
                    dotlottie_state_machine_post_click(sm, x, y);

                    printf("🖱 Click at (%.0f, %.0f) px\n", x, y);
                }
            }
        }

        // Poll DotLottie player events
        struct dotlottieDotLottiePlayerEvent player_event;
        while (dotlottie_poll_event(player, &player_event) == 1) {
            // We can log player events if needed
            if (player_event.event_type == Complete) {
                log_event(&event_log, "✓ Animation complete");
            }
        }

        // Poll State Machine events (only if SM exists)
        if (sm != NULL) {
            struct dotlottieStateMachineEvent sm_event;
            while (dotlottie_state_machine_poll_event(sm, &sm_event) == 1) {
            switch (sm_event.event_type) {
                case StateMachineStart:
                    snprintf(temp_msg, sizeof(temp_msg), "SM: Start");
                    log_event(&event_log, temp_msg);
                    printf("📊 State Machine: Start\n");
                    break;

                case StateMachineStop:
                    snprintf(temp_msg, sizeof(temp_msg), "SM: Stop");
                    log_event(&event_log, temp_msg);
                    printf("📊 State Machine: Stop\n");
                    break;

                case StateMachineTransition: {
                    char from[256] = {0}, to[256] = {0};
                    strncpy(from, sm_event.data.strings.str1, 255);
                    strncpy(to, sm_event.data.strings.str2, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: %s -> %s", from, to);
                    log_event(&event_log, temp_msg);
                    printf("🔄 Transition: %s -> %s\n", from, to);
                    break;
                }

                case StateMachineStateEntered: {
                    char state[256] = {0};
                    strncpy(state, sm_event.data.strings.str1, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: Entered '%s'", state);
                    log_event(&event_log, temp_msg);
                    printf("➡ State entered: %s\n", state);
                    break;
                }

                case StateMachineStateExit: {
                    char state[256] = {0};
                    strncpy(state, sm_event.data.strings.str1, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: Exit '%s'", state);
                    log_event(&event_log, temp_msg);
                    printf("⬅ State exit: %s\n", state);
                    break;
                }

                case StateMachineCustomEvent: {
                    char message[256] = {0};
                    strncpy(message, sm_event.data.strings.str1, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: Event '%s'", message);
                    log_event(&event_log, temp_msg);
                    printf("📨 Custom event: %s\n", message);
                    break;
                }

                case StateMachineError: {
                    char error[256] = {0};
                    strncpy(error, sm_event.data.strings.str1, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM ERROR: %s", error);
                    log_event(&event_log, temp_msg);
                    printf("❌ Error: %s\n", error);
                    break;
                }

                case StateMachineStringInputChange: {
                    char name[256], old_val[256], new_val[256];
                    strncpy(name, sm_event.data.strings.str1, 255);
                    strncpy(old_val, sm_event.data.strings.str2, 255);
                    strncpy(new_val, sm_event.data.strings.str3, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: %s='%s'", name, new_val);
                    log_event(&event_log, temp_msg);
                    printf("🔤 String input '%s': '%s' -> '%s'\n", name, old_val, new_val);
                    break;
                }

                case StateMachineNumericInputChange: {
                    char name[256];
                    strncpy(name, sm_event.data.numeric.name, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: %s=%.2f",
                             name, sm_event.data.numeric.new_value);
                    log_event(&event_log, temp_msg);
                    printf("🔢 Numeric input '%s': %.2f -> %.2f\n",
                           name, sm_event.data.numeric.old_value,
                           sm_event.data.numeric.new_value);
                    break;
                }

                case StateMachineBooleanInputChange: {
                    char name[256];
                    strncpy(name, sm_event.data.boolean.name, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: %s=%s",
                             name, sm_event.data.boolean.new_value ? "true" : "false");
                    log_event(&event_log, temp_msg);
                    printf("✓ Boolean input '%s': %s -> %s\n",
                           name,
                           sm_event.data.boolean.old_value ? "true" : "false",
                           sm_event.data.boolean.new_value ? "true" : "false");
                    break;
                }

                case StateMachineInputFired: {
                    char name[256];
                    strncpy(name, sm_event.data.strings.str1, 255);
                    snprintf(temp_msg, sizeof(temp_msg), "SM: Fired '%s'", name);
                    log_event(&event_log, temp_msg);
                    printf("🔥 Input fired: %s\n", name);
                    break;
                }
            }
        }
        }  // End of if (sm != NULL) for polling

        // Tick - use SM tick if active, otherwise player tick
        Uint32 current_tick = SDL_GetTicks();
        if (current_tick - last_tick >= 16) {
            if (sm != NULL) {
                dotlottie_state_machine_tick(sm);
            } else {
                dotlottie_tick(player);
            }
            last_tick = current_tick;
        }

        // Render
        SDL_SetRenderDrawColor(renderer, 26, 26, 26, 255);
        SDL_RenderClear(renderer);

        // Draw animation
        SDL_UpdateTexture(texture, NULL, buffer, ANIMATION_SIZE * sizeof(Uint32));
        SDL_RenderCopy(renderer, texture, NULL, &anim_rect);

        // Draw animation border
        SDL_SetRenderDrawColor(renderer, 100, 100, 100, 255);
        SDL_RenderDrawRect(renderer, &anim_rect);

        // Draw UI
        if (font) {
            draw_button(renderer, font, &buttons.start_sm, sm != NULL);
            draw_button(renderer, font, &buttons.stop_sm, false);

            // Draw event log
            draw_event_log(renderer, font, &event_log, UI_PADDING,
                          ANIMATION_SIZE + UI_PADDING * 2 + BUTTON_HEIGHT + 20);
        }

        SDL_RenderPresent(renderer);
        SDL_Delay(1);
    }

    ret = 0;

quit:
    // Release state machine first (if exists)
    if (sm)
        dotlottie_state_machine_release(sm);
    // Then destroy player
    if (player)
        dotlottie_destroy(player);
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
