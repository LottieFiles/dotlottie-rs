.PHONY: all run clean

# Use the new dotlottie-rs c_api bindings
DOTLOTTIE_PLAYER_LIB_PATH=../../target/release
DOTLOTTIE_PLAYER_HEADER_PATH=../../build
DOTLOTTIE_PLAYER_LIB=dotlottie_rs

SDL2_CFLAGS=$(shell sdl2-config --cflags)
SDL2_LIBS=$(shell sdl2-config --libs) -lSDL2_image

all: demo-player

demo-player: demo-player.c
	$(CC) -g demo-player.c -I$(DOTLOTTIE_PLAYER_HEADER_PATH) $(SDL2_CFLAGS) $(SDL2_LIBS) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -o $@


run: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run: demo-player
	@./demo-player $(ANIMATION_PATH)

clean:
	rm -rf demo-player