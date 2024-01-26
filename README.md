# dotLottie Rust

![CI](https://github.com/LottieFiles/dotlottie-rs/workflows/release/badge.svg)
![GitHub contributors](https://img.shields.io/github/contributors/LottieFiles/dotlottie-rs)
![GitHub](https://img.shields.io/github/license/LottieFiles/dotlottie-rs)

<p align="center">
  <img src="https://user-images.githubusercontent.com/23125742/201124166-c2a0bc2a-018b-463b-b291-944fb767b5c2.png" />
</p>

<h1 align="center">dotLottie Rust</h1>

This is the Rust implementation of the dotLottie player and its related tools. It utilizes uniffi-rs to generate FFI bindings for Kotlin, Swift, and WebAssembly (WASM). Checkout the releases page for the latest binaries.

## What is dotLottie?

dotLottie is an open-source file format that aggregates one or more Lottie files and their associated resources into a single file. They are ZIP archives compressed with the Deflate compression method and carry the file extension of ".lottie".

[Learn more about dotLottie](https://dotlottie.io/).

## Contents

- [crates](#crates)
- [Development](#development)
- [License](#license)

## crates

- [dotlottie-rs](./dotlottie-rs): The core library for dotLottie native players
- [dotlottie-fms](./dotlottie-fms): The dotLottie file management system library
- [dotlottie-ffi](./dotlottie-ffi): The FFI bindings for dotLottie core player to kotlin, swift and wasm
- [demo-player](./demo-player): A demo player for dotLottie written in Rust

## Development

### Build Instructions

To build for all target platforms, it would be best to use a Mac. You will also need GNU `make`
installed, at a bare minimum. To ensure that your local machine has all the other necessary
tools installed to build the project, run the following from the root of the repo:

```bash
$ make mac-setup
```

### Performing builds

Builds can be performed for the following groups of targets:

- `android`
- `apple`
- `WASM`

For `android` and `apple`, builds will be performed for all supported architectures, whereas
for `WASM`, only a single target will be built. These names refer to Makefile targets that can be
used to build them. For example, to build all `android` targets, execute the following:

```bash
$ make android
```

To build all targets, execute the following:

```bash
$ make all
```

### Other useful targets

- `demo-player`: Build the demo player
- `clean`: Cleanup rust build artifacts
- `distclean`: Cleanup ALL build artifacts

More information can be found by using the `help` target:

```bash
$ make help
```

### Release Process

Manually execute the `Create Release PR` Github Action workflow to create a release PR. This will
include all changes since the last release. This repo uses [changesets](https://github.com/changesets/changesets)
to determine the new release version. The [knope](https://github.com/knope-dev/knope) tool can be installed locally
and used to simply the creation of changeset files.

The release PR should be checked for correctness and then merged. Once that is done, the `Release`
Github Actions workflow will be started automatically to do the work of actually creating the new
release and building & uploading the related release artifacts.

### License

[MIT](LICENSE) © [LottieFiles](https://www.lottiefiles.com)
