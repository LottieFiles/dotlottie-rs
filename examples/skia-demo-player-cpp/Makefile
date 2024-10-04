.PHONY: all run debug clean

DOTLOTTIE_PLAYER_LIB_PATH=../../dotlottie-ffi/target/release
DOTLOTTIE_PLAYER_LIB=dotlottie_player

SDL2_CFLAGS=$(shell sdl2-config --cflags)
SDL2_LIBS=$(shell sdl2-config --libs)

BUILD_PLATFORM := $(shell uname -s | tr '[:upper:]' '[:lower:]')
MAC_BUILD_PLATFORM := darwin
ifeq ($(BUILD_PLATFORM),$(MAC_BUILD_PLATFORM))
	FRAMEWORKS := -framework CoreText -framework CoreGraphics -framework Foundation -framework CoreServices
endif

all: demo-player

demo-player: skia main.cpp
	$(CXX) -g --std=c++20 -Iskia -Lskia/out/Static -L$(DOTLOTTIE_PLAYER_LIB_PATH) main.cpp \
		$(SDL2_CFLAGS) $(SDL2_LIBS) -l$(DOTLOTTIE_PLAYER_LIB) -lskia -ldng_sdk \
		$(FRAMEWORKS) -o $@

run: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
run: demo-player
	@./demo-player $(ANIMATION_PATH)

debug: export LD_LIBRARY_PATH = $(DOTLOTTIE_PLAYER_LIB_PATH)
debug: demo-player
	@gdb demo-player

skia:
	git clone git@github.com:google/skia.git

skia-build: skia
	cd skia && \
		python3 tools/git-sync-deps && \
		python3 bin/fetch-ninja && \
		bin/gn gen out/Static \
			--args='is_official_build=true skia_use_system_libjpeg_turbo=false skia_use_system_libpng=false skia_use_system_libwebp=false skia_use_system_zlib=false skia_use_icu=false skia_use_icu=false skia_use_harfbuzz=false skia_use_freetype=false' && \
		ninja -C out/Static

clean:
	rm -rf demo-player
