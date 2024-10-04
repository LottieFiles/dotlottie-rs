.PHONY: all run debug clean

DOTLOTTIE_PLAYER_LIB_PATH=../../dotlottie-ffi/target/debug
DOTLOTTIE_PLAYER_LIB=dotlottie_player

all: demo-player

demo-player: skia main.cpp
	$(CXX) -g -Iskia -Lskia/out/Static -L$(DOTLOTTIE_PLAYER_LIB_PATH) main.cpp \
		-lSDL2 -l$(DOTLOTTIE_PLAYER_LIB) -lskia -lpng -ljpeg -lz -lfreetype -lwebp -lwebpdemux \
		-o $@

run: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run: demo-player
	@./demo-player $(ANIMATION_PATH)

debug: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
debug: demo-player
	@gdb demo-player

skia:
	git clone git@github.com:google/skia.git
	cd skia && \
		python3 tools/git-sync-deps && \
		python3 bin/fetch-ninja && \
		bin/gn gen out/Static --args='is_official_build=true' && \
		ninja -C out/Static

clean:
	rm -rf demo-player
