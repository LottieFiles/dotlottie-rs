## Getting up an running with Thorvg


### 1. Cloning the repo

```bash
git clone git@github.com:thorvg/thorvg.git
```

### 2. Building and installing

```bash
meson . builddir -Dbindings=capi
ninja -C builddir install
```

This will build and install the project with C bindings - This is needed for Rust.

### 3. Assure that the correct header and library files are installed

You should have

```bash
cat /usr/local/include/thorvg_capi.h
```

And 

```bash
cat /usr/local/lib/libthorvg.a
cat /usr/local/lib/libthorvg.dylib
```

### 4. Cargo run

Inside this project run

```bash
cargo run
```

This will use bindgen to create bindings inside 'bindings.rs' which will be in the build folder.

It will then build the project, you should have access to the C Api methods.

# Important remarks

The Thorvg C api is different than the C++ api. The header files are also outdated and doesn't every feature of the library mapped out, this include Lottie support.