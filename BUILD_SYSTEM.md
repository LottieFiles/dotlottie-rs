## Build System

The build system uses GNU `make` to build all artifacts for `android`, `apple`, and `wasm`. This
documentation provides some low-level implementation details relating to these builds. You can use
this information to better understand how the builds work in order to extend the build system and
make changes to it.

The build process works as follows:

1. If not already built, perform a build of `Thorvg` and it's native dependencies for the local machine architecture
2. For each target architecture, e.g. `aarch64-apple-darwin`, build `Thorvg` and, for Apple and Android targets, its native dependencies
3. Generate `uniffi` bindings for the target platform, e.g. Android, by first building `uniffi-bindgen`. This relies on a local architecture build of `Thorvg`
    - For WASM, instead use `uniffi-bindgen-cpp` to generate C++ bindings
4. Build the Rust `dotlottie-player` library
5. Put together the final release artifacts for each platform, i.e. Android, Apple, etc. and populate the `release` directory

### Define blocks

`define` blocks act like functions and are used for various purposes, such as to dynamically create
sections of the Makefile, create output files, etc. The sections that follow provide details
of each of these blocks.

When these blocks are referenced, they will usually be called with a mixture of top-level make
variables and target-specific variables. Examples of target-specific variables are as follows:

```
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): export PKG_CONFIG_PATH := $(PWD)/$(LOCAL_ARCH_LIB_DIR)/pkgconfig:$(PWD)/$(LOCAL_ARCH_LIB64_DIR)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): THORVG_DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$(THORVG)
```

Both of these variables are being given values in the specific case of the build for
`$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE)`. In the first line, the value will
also be exported to the shell environment being running the associated make recipe. In the case of
the second line, the variable value will only be visible within the context of the make file.

#### Thorvg

These blocks are used to build the external dependencies for `Thorvg`, which are only used
for Android and Apple build targets:

- `SETUP_CMAKE`: Perform a CMake setup for `Thorvg`'s dependencies
- `ANDROID_CMAKE_TOOLCHAIN_FILE`: Creates a `CMake` Toolchain file used to build `Thorvg`'s dependencies
- `CMAKE_BUILD`: Performs a `CMake` build as per a `CMake` specification
- `CMAKE_MAKE_BUILD`: Performs a make build as per a `CMake` specification
- `NEW_LOCAL_ARCH_CMAKE_BUILD`: Used to setup local architecture builds, i.e. for the local build machine
- `NEW_ANDROID_CMAKE_BUILD`: Defines a `CMake` build for `Thorvg` dependencies for Android
- `NEW_APPLE_CMAKE_BUILD`: Defines a `CMake` build for `Thorvg` dependencies for Apple

The following define blocks are used to setup `Meson` cross files for use with the `Thorvg` build. They are
parameterized using Makefile variables, and their output is not yet written to file:

- `ANDROID_CROSS_FILE`: Defines an Android cross file to be used with `Meson`
- `APPLE_CROSS_FILE`: Defines an Apple cross file to be used with `Meson`
- `WASM_CROSS_FILE`: Defines an WASM cross file to be used with `Meson`

These blocks use the previous ones and output the result to a file:

- `NEW_ANDROID_CROSS_FILE`: Creates an Android cross file for use with `Thorvg`
- `NEW_APPLE_CROSS_FILE`: Creates an Apple cross file for use with `Thorvg`
- `NEW_WASM_CROSS_FILE`: Creates a WASM cross file for use with `Thorvg`

The following blocks are used to build `Thorvg`:

- `SETUP_MESON`: Runs `Meson` to setup a build using `Ninja`
- `NINJA_BUILD`: Performs a `Ninja` build, as per a `Meson` build specification
- `NEW_THORVG_BUILD`: Defines a new build of `Thorvg`

Finally, these blocks build on the previous ones to perform the builds for `Thorvg` and all of its
dependencies:

- `NEW_ANDROID_DEPS_BUILD`: Performs the native builds required for Android
- `NEW_APPLE_DEPS_BUILD`: Performs the native builds required for Apple
- `NEW_WASM_DEPS_BUILD`: Performs the native builds required for WASM

#### Rust

`Cargo` is used to build `uniffi-bindgen` and the `dotlottie-player` library:

- `CARGO_BUILD`: Performs a `Cargo` build for Rust code

The following blocks are used to create `uniffi` bindings:

- `UNIFFI_BINDINGS_BUILD`: Creates UniFFI bindings for a specified language
- `UNIFFI_BINDINGS_CPP_BUILD`: Creates UniIFFI bindings for C++, used for WASM builds

#### Releases

The produce a release for Android, we must build up a directory containing all relevant
architecture builds, the `uniffi` files for Kotlin, and other supporting files.

- `ANDROID_RELEASE`: Compiles the final artifacts for an Android release 

For Apple, we must:

1. Build a Lipo library
2. Create a Framework
3. Create an XC Framework

The following define blocks are used to achieve this:

- `LIPO_CREATE`: Creates a Lipo library artifacts
- `APPLE_MODULE_MAP_FILE`: Creates a Module Map file for an Apple release
- `CREATE_FRAMEWORK`: Creates a Framework
- `NEW_APPLE_FRAMEWORK`: Creates the Framework and XC Framework
- `APPLE_RELEASE`: Compiles the final artifacts for an Apple release

For WASM builds, we must compile the `uniffi-bindgen-cpp` generated bindings with the manually
maintained `emscripten` C++ bindings, along with the `dotlottie-player` rust library to build
the final release artifacts.

- `WASM_MESON_BUILD_FILE`: Creates the `Meson` file used to build the WASM release artifacts
- `SETUP_WASM_MESON`: Runs `Meson` to setup a WASM build using `Ninja`
- `WASM_RELEASE`: Compiles the final artifacts for a WASM release

#### Top-level

Each build operation heavily relies on Makefile variables, which allows for build data to be defined
in a single place and reduces duplication. The variables for each build target are setup using the
following blocks:

- `NEW_BUILD_TARGET`: Defines to required make variables for a new build target
- `NEW_APPLE_TARGET`: Defines additional variables required for Apple builds

These previous blocks require access to the name of the target in SCREAMING_SNAKE case, in order
to aid in the definition of the new variables. This is achieved using the following blocks:

- `DEFINE_TARGET`: Simple helper function to define a new target
- `DEFINE_APPLE_TARGET`: Simple helper function to define a new apple target

After defining a target, the following top-level blocks perform all the necessary actions for a
particular build type:

- `NEW_ANDROID_BUILD`: Performs the Rust builds and release actions for Android
- `NEW_APPLE_BUILD`: Performs the Rust builds and release actions for Apple
- `NEW_WASM_BUILD`: Performs the Rust/C++ builds and release actions for WASM

#### Utilities

The following are general utility blocks:

- `TARGET_PREFIX`: Simple helper function to convert `cucumber-case` to `SCREAMING_SNAKE`
- `CREATE_OUTPUT_FILE`: General utility to create an output file

### Delayed variable expansion

In certain define blocks, such as `NEW_BUILD_TARGET`, you will notice the use of a double-dollar (`$$`)
expansions, such as:

```
$2_THORVG_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(THORVG)
```

This is for the purpose of using the block that contains this code with the make `eval` function, which
allows for dynamically creating sections of the Makefile. This greatly reduces the amount of repetition in
the Makefile, and thus maintenance overhead, at the cost of a small amount of complexity.

When a `define` block is called, variables references contained within it are expanded, and
this behaviour is usually what you would want to happen outside the context of `eval`. However, when
using `eval`, we may want certain variables to be expanded later by `eval` instead.

In the example given above, we want `$2_THORVG_DEP_BUILD_DIR` to be expanded into the name of a variable
to be created. As `$2` in this case is defined as the SCREAMING_SNAKE_CASE version of the current target
architecture, and will be have a value such as `AARCH64_LINUX_ANDROID`, the line above will be expanded to
something like the following, _before_ being passed to `eval`:

```
AARCH64_LINUX_ANDROID_THORVG_DEP_BUILD_DIR := $(AARCH64_LINUX_ANDROID_DEPS_BUILD_DIR)/thorvg
```

Here we can see that all the variables have been expanded, however, one of them still looks like a
variable. This one will be expanded by `eval`, thus giving us the ability to dereference the
`AARCH64_LINUX_ANDROID_DEPS_BUILD_DIR` variable only in the context of the `eval`. This technique
is used fairly heavily throughout the Makefile.

To get a view of what these evaluated sections of the Makefile look like, you can try replacing
any `eval` call with `info`, and then running `make` without any arguments. This will display the
result of the expansion without evaluting it, which can be useful for debugging.

### Submodule management

This repo uses git submodules for its external dependencies. Though these will normally be setup
for you when running `make mac-setup`, it can sometimes be useful to run `make deps` manually as
well.

If the version of a submodule, such as for `Thorvg`, is updated, when you pull this change your
reference to the submodule will be updated, however, your local clone of the submodule
would still point to the old commit. To bring your local copy into line with the checked in
commit of the submodule, run `make deps`.

### Incremental builds

Performing a `make all` and building all possible targets can take a long time when performed from
scratch. However, after the initial build, the next `make all` build operation will be significantly
faster, as all previous build files, such as for `Thorvg`, will already be available.

#### Cleanup

After building all artifacts, running `make distclean` will wipe out everything and return you to a
clean repo, and is usually not what you want to do. Run `make clean` instead to just remove Rust
build files. In most cases, this is also not required, and you can simply rebuild the target you are
working with to perform an incremental build.

There a small quirk with the `zlib` dependency build, which makes its submodule clone appear dirty
after a build. This does not cause any real problems, but can show up in git as an unncessary change.
If this bothers you, run `make clean-build`.
