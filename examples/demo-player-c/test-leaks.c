#include "dotlottie_runtime.h"
#include <stdio.h>

int main() {
  printf("Simple memory leak test\n");
  printf("======================\n\n");

  // Test 1: Create and destroy player
  printf("Test 1: Create player with default config...\n");
  DotLottieConfig config;
  dotlottie_init_config(&config);

  DotLottieRuntime *player = dotlottie_new_player(&config);
  if (!player) {
    fprintf(stderr, "Failed to create player\n");
    return 1;
  }
  printf("  ✓ Player created\n");

  printf("Test 2: Destroy player...\n");
  dotlottie_destroy(player);
  printf("  ✓ Player destroyed\n\n");

  printf("Test complete. If no leaks reported, memory is clean!\n");
  return 0;
}
