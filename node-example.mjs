import createDotLottiePlayerModule from "./release/wasm/dotlottie_player.js";
import fs from "fs";
import path from "path";

const wasmBinary = fs.readFileSync(
  path.join("./release/wasm/dotlottie_player.wasm")
);

const Module = await createDotLottiePlayerModule({
  wasmBinary,
});

const DOTLOTTIE_SUCCESS = Module._dotlottie_success();

function withString(str, callback) {
  const size = Module.lengthBytesUTF8(str) + 1;
  const ptr = Module._malloc(size);
  Module.stringToUTF8(str, ptr, size);
  try {
    return callback(ptr);
  } finally {
    Module._free(ptr);
  }
}

function getBufferPtr(playerPtr) {
  const resultPtr = Module._malloc(4);
  Module._dotlottie_buffer_ptr(playerPtr, resultPtr);
  const bufferPtr = Module.getValue(resultPtr, "i32");
  Module._free(resultPtr);
  return bufferPtr;
}

function createPlayerWithConfig(options = {}) {
  const configPtr = Module._dotlottie_config_new();

  if (options.autoplay !== undefined)
    Module._dotlottie_config_set_autoplay(configPtr, options.autoplay);
  if (options.loopAnimation !== undefined)
    Module._dotlottie_config_set_loop_animation(configPtr, options.loopAnimation);
  if (options.backgroundColor !== undefined)
    Module._dotlottie_config_set_background_color(configPtr, options.backgroundColor);

  const playerPtr = Module._dotlottie_new_player(configPtr);
  Module._dotlottie_config_free(configPtr);

  return playerPtr;
}

const WIDTH = 200;
const HEIGHT = 200;

const playerPtr = createPlayerWithConfig({
  backgroundColor: 0xff009aff,
});

if (!playerPtr) {
  console.log("Failed to create player");
  process.exit(1);
}

const data = await fetch(
  "https://lottie.host/647eb023-6040-4b60-a275-e2546994dd7f/zDCfp5lhLe.json"
).then((res) => res.text());

const loaded = withString(data, (dataPtr) =>
  Module._dotlottie_load_animation_data(playerPtr, dataPtr, WIDTH, HEIGHT)
);

if (loaded !== DOTLOTTIE_SUCCESS) {
  console.log("Failed to load animation data");
  process.exit(1);
}

Module._dotlottie_set_frame(playerPtr, 10.0);
const rendered = Module._dotlottie_render(playerPtr);

if (rendered !== DOTLOTTIE_SUCCESS) {
  console.log("Failed to render");
  process.exit(1);
}

const bufferPtr = getBufferPtr(playerPtr);
const bufferLen = WIDTH * HEIGHT * 4;
const frameBuffer = new Uint8ClampedArray(
  Module.HEAPU8.buffer,
  bufferPtr,
  bufferLen
);

const bmpBuffer = createBMP(WIDTH, HEIGHT, frameBuffer);
fs.writeFileSync("./output.bmp", bmpBuffer);

console.log("Successfully rendered frame to output.bmp");

Module._dotlottie_destroy(playerPtr);


function createBMP(width, height, frameBuffer) {
  const bmpDataSize = width * height * 4;
  const headerSize = 54;
  const fileSize = bmpDataSize + headerSize;
  const bmpBuffer = Buffer.alloc(fileSize);

  // Bitmap file header
  bmpBuffer.write("BM", 0);
  bmpBuffer.writeInt32LE(fileSize, 2);
  bmpBuffer.writeInt32LE(headerSize, 10);

  // DIB header
  bmpBuffer.writeInt32LE(40, 14);
  bmpBuffer.writeInt32LE(width, 18);
  bmpBuffer.writeInt32LE(-height, 22); // Negative for top-down bitmap
  bmpBuffer.writeInt16LE(1, 26);
  bmpBuffer.writeInt16LE(32, 28);
  bmpBuffer.writeInt32LE(0, 30);

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
