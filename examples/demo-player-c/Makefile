.PHONY: all run debug

DOTLOTTIE_PLAYER_LIB_PATH=../../dotlottie-ffi/target/release
DOTLOTTIE_PLAYER_LIB=dotlottie_player

SDL2_CFLAGS=$(shell sdl2-config --cflags)
SDL2_LIBS=$(shell sdl2-config --libs)

all: demo-player

demo-player: main.c
	$(CC) -g main.c $(SDL2_CFLAGS) $(SDL2_LIBS) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -o $@

run: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run: demo-player
	@./demo-player $(ANIMATION_PATH)

debug: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
debug: demo-player
	@gdb demo-player

clean:
	rm -rf demo-player
