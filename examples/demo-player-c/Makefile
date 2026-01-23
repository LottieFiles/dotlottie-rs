.PHONY: all run debug

# Use the new dotlottie-rs c_api bindings
DOTLOTTIE_PLAYER_LIB_PATH=../../release/native/dotlottie-player/lib/
DOTLOTTIE_PLAYER_HEADER_PATH=../../release/native/dotlottie-player/include/
DOTLOTTIE_PLAYER_LIB=dotlottie_player

SDL2_CFLAGS=$(shell sdl2-config --cflags)
SDL2_LIBS=$(shell sdl2-config --libs) -lSDL2_ttf

all: demo-player demo-player-perf demo-player-events demo-player-state-machine

demo-player: main.c
	$(CC) -g main.c -I$(DOTLOTTIE_PLAYER_HEADER_PATH) $(SDL2_CFLAGS) $(SDL2_LIBS) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -o $@

demo-player-perf: demo-player-perf.c
	$(CC) -g demo-player-perf.c -I$(DOTLOTTIE_PLAYER_HEADER_PATH) $(SDL2_CFLAGS) $(SDL2_LIBS) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -lm -o $@

demo-player-events: demo-player-events.c
	$(CC) -g demo-player-events.c -I$(DOTLOTTIE_PLAYER_HEADER_PATH) $(SDL2_CFLAGS) $(SDL2_LIBS) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -o $@

demo-player-state-machine: demo-player-state-machine.c
	$(CC) -g demo-player-state-machine.c -I$(DOTLOTTIE_PLAYER_HEADER_PATH) $(SDL2_CFLAGS) $(SDL2_LIBS) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -o $@

test-leaks: test-leaks.c
	$(CC) -g test-leaks.c -I$(DOTLOTTIE_PLAYER_HEADER_PATH) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -o $@

run: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run: demo-player
	@./demo-player $(ANIMATION_PATH)

run-perf: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run-perf: demo-player-perf
	@if [ -z "$(NUM_ANIMATIONS)" ]; then \
		./demo-player-perf $(ANIMATION_PATH); \
	else \
		./demo-player-perf $(ANIMATION_PATH) $(NUM_ANIMATIONS); \
	fi

test-leaks: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
test-leaks: export MallocStackLogging = 1
test-leaks: test-leaks
	@echo "Running memory leak test..."
	@leaks --atExit -- ./test-leaks

run-events: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run-events: demo-player-events
	@./demo-player-events $(ANIMATION_PATH)

run-state-machine: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run-state-machine: demo-player-state-machine
	@./demo-player-state-machine $(ANIMATION_PATH)

debug: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
debug: demo-player
	@gdb demo-player

clean:
	rm -rf demo-player demo-player-perf demo-player-events demo-player-state-machine


