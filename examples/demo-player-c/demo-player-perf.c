#include <SDL.h>
#include <SDL_pixels.h>
#include <SDL_ttf.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <sys/time.h>
#include <sys/resource.h>

#ifdef __APPLE__
#include <mach/mach.h>
#include <mach/task_info.h>
#endif

#include "dotlottie_player.h"

#define MAX_ANIMATIONS 1040
#define ANIMATION_SIZE 200

typedef struct {
    DotLottiePlayer *player;
    const uint32_t *buffer;
    SDL_Texture *texture;
    int x;
    int y;
    int width;
    int height;
} AnimationInstance;

typedef struct {
    double fps;
    double avg_frame_time_ms;
    double total_render_time_ms;
    int frame_count;
    struct timeval last_fps_update;
    long memory_mb;
    double cpu_percent;
} PerformanceMetrics;

void usage(char *app) {
    fprintf(stderr, "usage: %s <animation-file> [num-animations]\n", app);
    fprintf(stderr, "  num-animations: number of animations to display (1-%d, default: 4)\n", MAX_ANIMATIONS);
    exit(1);
}

double get_time_ms() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (tv.tv_sec * 1000.0) + (tv.tv_usec / 1000.0);
}

long get_memory_usage_mb() {
#ifdef __APPLE__
    // On macOS, use task_info to get current memory usage (matches Activity Monitor)
    struct mach_task_basic_info info;
    mach_msg_type_number_t size = MACH_TASK_BASIC_INFO_COUNT;
    kern_return_t kr = task_info(mach_task_self(), MACH_TASK_BASIC_INFO, (task_info_t)&info, &size);
    if (kr == KERN_SUCCESS) {
        return info.resident_size / (1024 * 1024); // Convert bytes to MB
    }
    return 0;
#else
    // On Linux, use getrusage
    struct rusage usage;
    if (getrusage(RUSAGE_SELF, &usage) == 0) {
        return usage.ru_maxrss / 1024; // Convert KB to MB
    }
    return 0;
#endif
}

void calculate_grid_layout(int num_animations, int window_width, int window_height,
                          int *cols, int *rows, int *anim_width, int *anim_height) {
    // Calculate grid dimensions
    *cols = (int)ceil(sqrt(num_animations));
    *rows = (num_animations + *cols - 1) / *cols;

    *anim_width = window_width / *cols;
    *anim_height = window_height / *rows;
}

void update_metrics(PerformanceMetrics *metrics, double frame_time_ms) {
    metrics->frame_count++;
    metrics->total_render_time_ms += frame_time_ms;
    metrics->avg_frame_time_ms = metrics->total_render_time_ms / metrics->frame_count;

    struct timeval now;
    gettimeofday(&now, NULL);
    double elapsed = (now.tv_sec - metrics->last_fps_update.tv_sec) +
                     (now.tv_usec - metrics->last_fps_update.tv_usec) / 1000000.0;

    if (elapsed >= 1.0) {
        metrics->fps = metrics->frame_count / elapsed;
        metrics->frame_count = 0;
        metrics->total_render_time_ms = 0;
        metrics->last_fps_update = now;
        metrics->memory_mb = get_memory_usage_mb();
    }
}

void render_text(SDL_Renderer *renderer, TTF_Font *font, const char *text,
                 int x, int y, SDL_Color color) {
    SDL_Surface *surface = TTF_RenderText_Blended(font, text, color);
    if (!surface) return;

    SDL_Texture *texture = SDL_CreateTextureFromSurface(renderer, surface);
    if (!texture) {
        SDL_FreeSurface(surface);
        return;
    }

    SDL_Rect rect = { x, y, surface->w, surface->h };
    SDL_RenderCopy(renderer, texture, NULL, &rect);

    SDL_DestroyTexture(texture);
    SDL_FreeSurface(surface);
}

void render_metrics(SDL_Renderer *renderer, TTF_Font *font, PerformanceMetrics *metrics,
                   int num_animations, int window_width) {
    if (!font) return;

    SDL_Color white = { 255, 255, 255, 255 };
    SDL_Color green = { 100, 255, 100, 255 };
    SDL_Color yellow = { 255, 255, 100, 255 };

    char text[256];
    int padding = 10;
    int x = 10;
    int y = 10;
    int line_height = 28;
    int num_lines = 5;

    // Calculate background size
    int bg_width = 250;
    int bg_height = num_lines * line_height + padding * 2;

    // Draw semi-transparent black background
    SDL_Rect bg_rect = { x - padding, y - padding, bg_width, bg_height };
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 180);
    SDL_RenderFillRect(renderer, &bg_rect);

    // Reset blend mode
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_NONE);

    // FPS
    snprintf(text, sizeof(text), "FPS: %.1f", metrics->fps);
    render_text(renderer, font, text, x, y, metrics->fps >= 55 ? green : yellow);
    y += line_height;

    // Frame Time
    snprintf(text, sizeof(text), "Frame Time: %.2f ms", metrics->avg_frame_time_ms);
    render_text(renderer, font, text, x, y, white);
    y += line_height;

    // Memory Usage
    snprintf(text, sizeof(text), "Memory: %ld MB", metrics->memory_mb);
    render_text(renderer, font, text, x, y, white);
    y += line_height;

    // Animation Count
    snprintf(text, sizeof(text), "Animations: %d", num_animations);
    render_text(renderer, font, text, x, y, white);
    y += line_height;

    // Resolution
    snprintf(text, sizeof(text), "Resolution: %dx%d", window_width,
             (int)(window_width * 9.0 / 16.0)); // Approximate based on aspect ratio
    render_text(renderer, font, text, x, y, white);
}

int main(int argc, char **argv) {
    SDL_Window *window = NULL;
    SDL_Renderer *renderer = NULL;
    SDL_Event e;
    TTF_Font *font = NULL;

    const char *animation_path;
    int num_animations = 4; // Default
    int ret = 0;

    AnimationInstance animations[MAX_ANIMATIONS] = {0};
    PerformanceMetrics metrics = {0};
    gettimeofday(&metrics.last_fps_update, NULL);

    // Parse arguments
    if (argc < 2) {
        usage(argv[0]);
    }

    animation_path = argv[1];
    if (access(animation_path, R_OK) != 0) {
        fprintf(stderr, "Invalid animation path\n\n");
        usage(argv[0]);
    }

    if (argc >= 3) {
        num_animations = atoi(argv[2]);
        if (num_animations < 1 || num_animations > MAX_ANIMATIONS) {
            fprintf(stderr, "Number of animations must be between 1 and %d\n", MAX_ANIMATIONS);
            return 1;
        }
    }

    // Initialize SDL
    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        fprintf(stderr, "Could not initialize SDL: %s\n", SDL_GetError());
        return 1;
    }

    // Initialize SDL_ttf
    if (TTF_Init() < 0) {
        fprintf(stderr, "Could not initialize SDL_ttf: %s\n", TTF_GetError());
        SDL_Quit();
        return 1;
    }

    // Use a reasonable default window size
    int window_width = 1200;
    int window_height = 900;

    // Calculate grid layout based on number of animations
    int cols, rows, anim_width, anim_height;
    calculate_grid_layout(num_animations, window_width, window_height,
                         &cols, &rows, &anim_width, &anim_height);

    // Create window
    window = SDL_CreateWindow("DotLottie Performance Test",
                              SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
                              window_width, window_height,
                              SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE | SDL_WINDOW_ALWAYS_ON_TOP | SDL_WINDOW_INPUT_FOCUS);
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

    // Get the display mode to determine refresh rate
    int display_index = SDL_GetWindowDisplayIndex(window);
    SDL_DisplayMode display_mode;
    if (SDL_GetCurrentDisplayMode(display_index, &display_mode) != 0) {
        fprintf(stderr, "Warning: Could not get display mode, defaulting to 60Hz\n");
        display_mode.refresh_rate = 60;
    }

    int refresh_rate = display_mode.refresh_rate;
    if (refresh_rate == 0) {
        refresh_rate = 60; // Default to 60Hz if unknown
    }
    double target_frame_time = 1000.0 / refresh_rate;

    // Load font (try common system font locations)
    const char *font_paths[] = {
        "/System/Library/Fonts/Helvetica.ttc",  // macOS
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",  // Linux
        "C:\\Windows\\Fonts\\arial.ttf"  // Windows
    };

    for (int i = 0; i < 3; i++) {
        font = TTF_OpenFont(font_paths[i], 20);
        if (font) break;
    }

    if (!font) {
        fprintf(stderr, "Warning: Could not load font, performance metrics will not be displayed\n");
    }

    // Create all animation instances
    for (int i = 0; i < num_animations; i++) {
        DotLottieConfig config;
        dotlottie_init_config(&config);
        config.loop_animation = true;
        config.autoplay = true;
        config.background_color = 0xffffffff;
        config.layout.fit = Contain;
        strcpy(config.marker.value, "idle_message");

        animations[i].player = dotlottie_new_player(&config);
        if (!animations[i].player) {
            fprintf(stderr, "Could not create player %d\n", i);
            ret = 1;
            goto quit;
        }

        ret = dotlottie_load_animation_path(animations[i].player, animation_path,
                                           anim_width, anim_height);
        if (ret != DOTLOTTIE_SUCCESS) {
            fprintf(stderr, "Could not load animation %d (error code: %d)\n", i, ret);
            fprintf(stderr, "Animation path: %s\n", animation_path);
            fprintf(stderr, "Animation dimensions: %dx%d\n", anim_width, anim_height);
            ret = 1;
            goto quit;
        }

        ret = dotlottie_buffer_ptr(animations[i].player, &animations[i].buffer);
        if (ret != DOTLOTTIE_SUCCESS) {
            fprintf(stderr, "Could not get buffer for animation %d\n", i);
            ret = 1;
            goto quit;
        }

        animations[i].texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_BGRA32,
                                                  SDL_TEXTUREACCESS_STREAMING,
                                                  anim_width, anim_height);
        if (!animations[i].texture) {
            fprintf(stderr, "Could not create texture %d\n", i);
            ret = 1;
            goto quit;
        }

        // Calculate position in grid
        int col = i % cols;
        int row = i / cols;
        animations[i].x = col * anim_width;
        animations[i].y = row * anim_height;
        animations[i].width = anim_width;
        animations[i].height = anim_height;
    }

    // Main loop
    int running = 1;
    while (running) {
        double frame_start = get_time_ms();

        // Process events
        while (SDL_PollEvent(&e) != 0) {
            if (e.type == SDL_QUIT) {
                running = 0;
            } else if (e.type == SDL_KEYDOWN) {
                if (e.key.keysym.sym == SDLK_q || e.key.keysym.sym == SDLK_ESCAPE) {
                    running = 0;
                }
            } else if (e.type == SDL_WINDOWEVENT) {
                if (e.window.event == SDL_WINDOWEVENT_SIZE_CHANGED) {
                    // Window was resized, recalculate layout
                    window_width = e.window.data1;
                    window_height = e.window.data2;

                    calculate_grid_layout(num_animations, window_width, window_height,
                                         &cols, &rows, &anim_width, &anim_height);

                    // Recreate all animations with new dimensions
                    for (int i = 0; i < num_animations; i++) {
                        // Destroy old resources
                        if (animations[i].texture) {
                            SDL_DestroyTexture(animations[i].texture);
                        }
                        if (animations[i].player) {
                            dotlottie_destroy(animations[i].player);
                        }

                        // Create new player with updated dimensions
                        DotLottieConfig config;
                        dotlottie_init_config(&config);
                        config.loop_animation = true;
                        config.autoplay = true;
                        config.background_color = 0xffffffff;
                        config.layout.fit = Contain;

                        animations[i].player = dotlottie_new_player(&config);
                        if (!animations[i].player) {
                            fprintf(stderr, "Could not recreate player %d\n", i);
                            running = 0;
                            break;
                        }

                        ret = dotlottie_load_animation_path(animations[i].player, animation_path,
                                                           anim_width, anim_height);
                        if (ret != DOTLOTTIE_SUCCESS) {
                            fprintf(stderr, "Could not reload animation %d\n", i);
                            running = 0;
                            break;
                        }

                        ret = dotlottie_buffer_ptr(animations[i].player, &animations[i].buffer);
                        if (ret != DOTLOTTIE_SUCCESS) {
                            fprintf(stderr, "Could not get buffer for animation %d\n", i);
                            running = 0;
                            break;
                        }

                        animations[i].texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_BGRA32,
                                                                  SDL_TEXTUREACCESS_STREAMING,
                                                                  anim_width, anim_height);
                        if (!animations[i].texture) {
                            fprintf(stderr, "Could not recreate texture %d\n", i);
                            running = 0;
                            break;
                        }

                        // Update position and size
                        int col = i % cols;
                        int row = i / cols;
                        animations[i].x = col * anim_width;
                        animations[i].y = row * anim_height;
                        animations[i].width = anim_width;
                        animations[i].height = anim_height;
                    }
                }
            }
        }

        // Clear screen
        SDL_SetRenderDrawColor(renderer, 40, 40, 40, 255);
        SDL_RenderClear(renderer);

        // Update and render all animations
        for (int i = 0; i < num_animations; i++) {
            // Tick the animation
            dotlottie_tick(animations[i].player);

            // Update texture
            SDL_UpdateTexture(animations[i].texture, NULL, animations[i].buffer,
                            animations[i].width * sizeof(Uint32));

            // Render to grid position
            SDL_Rect dst_rect = {
                animations[i].x,
                animations[i].y,
                animations[i].width,
                animations[i].height
            };
            SDL_RenderCopy(renderer, animations[i].texture, NULL, &dst_rect);

            // Draw border
            SDL_SetRenderDrawColor(renderer, 100, 100, 100, 255);
            SDL_RenderDrawRect(renderer, &dst_rect);
        }

        // Render performance metrics on top
        render_metrics(renderer, font, &metrics, num_animations, window_width);

        SDL_RenderPresent(renderer);

        // Update metrics
        double frame_time = get_time_ms() - frame_start;
        update_metrics(&metrics, frame_time);

        // Cap at monitor's refresh rate
        if (frame_time < target_frame_time) {
            SDL_Delay((Uint32)(target_frame_time - frame_time));
        }
    }

quit:
    // Clean up animations
    for (int i = 0; i < num_animations; i++) {
        if (animations[i].texture) {
            SDL_DestroyTexture(animations[i].texture);
        }
        if (animations[i].player) {
            dotlottie_destroy(animations[i].player);
        }
    }

    if (font) {
        TTF_CloseFont(font);
    }
    if (renderer) {
        SDL_DestroyRenderer(renderer);
    }
    if (window) {
        SDL_DestroyWindow(window);
    }
    TTF_Quit();
    SDL_Quit();

    return ret;
}
