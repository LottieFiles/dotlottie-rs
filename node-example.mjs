import createDotLottiePlayerModule from "./release/wasm/DotLottiePlayer.mjs";
import fs from "fs";
import path from "path";

const wasmBinary = fs.readFileSync(
  path.join("./release/wasm/DotLottiePlayer.wasm")
);

const Module = await createDotLottiePlayerModule({
  wasmBinary,
});

const WIDTH = 200;
const HEIGHT = 200;

const dotLottie = new Module.DotLottiePlayer({
  autoplay: false,
  loop_animation: false,
  mode: Module.Mode.values[1],
  speed: 1,
  use_frame_interpolation: false,
});

const data = await fetch(
  "https://lottie.host/647eb023-6040-4b60-a275-e2546994dd7f/zDCfp5lhLe.json"
).then((res) => res.text());

const loaded = dotLottie.load_animation_data(data, WIDTH, HEIGHT);

if (!loaded) {
  console.log("failed to load animation data");
}

dotLottie.set_frame(10.0);
const rendered = dotLottie.render();

if (!rendered) {
  console.log("failed to render");
}

const frameBuffer = dotLottie.buffer();

const bmpBuffer = createBMP(WIDTH, WIDTH, frameBuffer);

fs.writeFileSync("./output.bmp", bmpBuffer);

// This is for demonstration purposes only. to avoid adding a dependency
function createBMP(width, height, frameBuffer) {
  // Each pixel in BMP is 4 bytes (BGRA)
  const bmpDataSize = width * height * 4;
  const headerSize = 54;
  const fileSize = bmpDataSize + headerSize;
  const bmpBuffer = Buffer.alloc(fileSize);

  // Bitmap file header
  bmpBuffer.write("BM", 0); // Signature
  bmpBuffer.writeInt32LE(fileSize, 2); // File size
  bmpBuffer.writeInt32LE(headerSize, 10); // Pixel data offset

  // DIB header
  bmpBuffer.writeInt32LE(40, 14); // DIB header size
  bmpBuffer.writeInt32LE(width, 18); // Width
  bmpBuffer.writeInt32LE(-height, 22); // Height (negative for top-down bitmap)
  bmpBuffer.writeInt16LE(1, 26); // Color planes
  bmpBuffer.writeInt16LE(32, 28); // Bits per pixel
  bmpBuffer.writeInt32LE(0, 30); // Compression (0 for none)

  // Convert RGBA to BGRA and write pixel data
  for (let i = 0; i < width * height; i++) {
    const rgbaIndex = i * 4;
    const bgraIndex = headerSize + i * 4;
    bmpBuffer[bgraIndex + 0] = frameBuffer[rgbaIndex + 2]; // Blue
    bmpBuffer[bgraIndex + 1] = frameBuffer[rgbaIndex + 1]; // Green
    bmpBuffer[bgraIndex + 2] = frameBuffer[rgbaIndex + 0]; // Red
    bmpBuffer[bgraIndex + 3] = frameBuffer[rgbaIndex + 3]; // Alpha
  }

  return bmpBuffer;
}
