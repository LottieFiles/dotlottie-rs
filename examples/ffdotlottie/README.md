# ffdotlottie

> **Note**: Currently works only on macOS. Linux and Windows support is in progress.

`ffdotlottie` is a command-line tool for converting `.lottie` or `.json` Lottie animation files into video format, using the `dotlottie-rs` and `video-rs` libraries.

To try it out quickly, an executable binary is available in the `bin/macos-arm64` folder.

```bash
./.bin/macos-arm64/ffdotlottie --input ./assets/cartoon.json --output ./output.mp4 --width 1920 --height 1080 --background-color "#00FFFFFF"
```

> This won't work if you're not on an M1 Mac. If you're on a different platform, you can build the tool using the instructions below.

## Features (In Progress)

- Load `.lottie` and `.json` animation files.
- Convert and encode animations into video format (H.264 YUV420p).
- Customize video dimensions (width/height).
- Specify a background color for the animation.

## Prerequisites

1. Ensure `ffmpeg` is installed on your system. Follow [these instructions](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building#dependencies) for installation.
2. Run `make mac-setup` in the project's root directory to install the necessary tools.
3. Build the native dependencies for `dotlottie-rs` by running `make aarch64-apple-darwin` if you're on macOS. This will compile the required native libraries, including ThorVG.

## Installation

```sh
cargo build --release
```

## Usage

After building the tool, run it by providing the required parameters:

```bash
./target/release/ffdotlottie --input <input_file> --output <output_file> --width <width> --height <height> [--background-color <hex_color_string>]
```

### Example

```bash
./target/release/ffdotlottie --input ./assets/cartoon.json --output ./output.mp4 --width 1920 --height 1080 --background-color "#FF0000FF"
```

### Arguments

- `--input`: Path to the `.lottie` or `.json` file.
- `--output`: Path where the output video will be saved.
- `--width`: Width of the output video in pixels.
- `--height`: Height of the output video in pixels.
- `--background-color` (optional): Background color of the animation (e.g., `#FFFF00FF`).

## Work in Progress

- Enhanced error handling and input validation.
- Multi-threading lottie/dotLottie rendering via ThorVG.
- Additional features like frame rate control and support for more encoding formats.
