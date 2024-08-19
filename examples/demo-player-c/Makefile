.PHONY: all run debug

DOTLOTTIE_PLAYER_LIB_PATH=../../dotlottie-ffi/target/debug
DOTLOTTIE_PLAYER_LIB=dotlottie_player

all: demo-player

demo-player: main.c
	gcc -ggdb main.c -L$(DOTLOTTIE_PLAYER_LIB_PATH) -lX11 -l$(DOTLOTTIE_PLAYER_LIB) -o $@

run: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run: demo-player
	@./demo-player $(ANIMATION_PATH)

debug: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
debug: demo-player
	@gdb demo-player
