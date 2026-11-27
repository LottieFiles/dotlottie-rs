.PHONY: all run clean

DOTLOTTIE_PLAYER_DIR=../../../release/native/dotlottie-player
DOTLOTTIE_PLAYER_LIB_PATH=$(DOTLOTTIE_PLAYER_DIR)/lib
DOTLOTTIE_PLAYER_HEADER_PATH=$(DOTLOTTIE_PLAYER_DIR)/include
DOTLOTTIE_PLAYER_LIB=dotlottie_player

SDL2_CFLAGS=$(shell sdl2-config --cflags)
SDL2_LIBS=$(shell sdl2-config --libs)

all: demo-player

demo-player: demo-player.c
	$(CC) -g demo-player.c -I$(DOTLOTTIE_PLAYER_HEADER_PATH) $(SDL2_CFLAGS) $(SDL2_LIBS) -L$(DOTLOTTIE_PLAYER_LIB_PATH) -l$(DOTLOTTIE_PLAYER_LIB) -Wl,-rpath,$(abspath $(DOTLOTTIE_PLAYER_LIB_PATH)) -o $@

run: demo-player
	@./demo-player $(ANIMATION_PATH)

clean:
	rm -rf demo-player demo-player.dSYM
