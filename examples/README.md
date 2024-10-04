# Examples

## C-API

### Running the examples

In order to try out the `demo-player-c` and `skia-demo-player-cpp` examples, you must first ensure that
`dotlottie-ffi` has been built, which can be done by executing `cargo build --release` in the `dotlottie-ffi`
directory, or by running `make native` from the root of the repo.

You can then build each of the demos by running `make` in their respective directories. You will
need the `SDL2` libraries and headers available on your system to perform the build. On Windows, you should
use a WSL2 environment, or build the sources manually.

For the Skia demo, you should run `make skia-build` in the `skia-demo-player-cpp` directory first.

On a Mac, the SDL2 dependencies can be installed through `brew`:

```Bash
brew install sdl2 sdl2_image
```

You can then run the examples using a command such as the following:

```Bash
make run ANIMATION_PATH=../demo-player/src/markers.json
```

### Using the API

When using the C bindings for `dotlottie-rs`, you must first create a `DotLottiePlayer`, which will then
be referenced in all further calls to the API. A `DotLottiePlayer` must be provided with a `DotLottieConfig`,
which can be created as follows:

```C
DotLottieConfig config;
dotlottie_init_config(&config);
```

The `config` can then be customized as required:

```C
config.loop_animation = true;
config.background_color = 0xffffffff;
```

The `config` is then be used to create a `DotLottiePlayer`:

```C
DotLottiePlayer *player = dotlottie_new_player(&config);
if (!player) {
    fprintf(stderr, "Could not create dotlottie player\n");
    return 1;
}
```

Further API calls can then be made using the returned `DotLottiePlayer` pointer:

```C
ret = dotlottie_load_animation_path(player, animation_path, WIDTH, HEIGHT);
if (ret != DOTLOTTIE_SUCCESS) {
    fprintf(stderr, "Could not load dotlottie animation file\n");
    return 1;
}
```

Every API call will return `DOTLOTTIE_SUCCESS` on success, and some other value on error. The set of
possible error return values can be found in the `dotlottie_player.h` header file.
