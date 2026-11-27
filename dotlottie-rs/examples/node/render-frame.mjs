// Node demo: render one frame with the WASM build and save it as a BMP.
//
//   make wasm                                             (from the repository root)
//   node dotlottie-rs/examples/node/render-frame.mjs [animation] [frame] [output.bmp]

import { readFileSync, writeFileSync } from "node:fs";

const WIDTH = 512;
const HEIGHT = 512;

const root = new URL("../../../", import.meta.url);
const [
  animationPath = new URL("dotlottie-rs/assets/animations/lottie/bouncy_ball.json", root),
  frame = "10",
  outputPath = "output.bmp",
] = process.argv.slice(2);

const { initSync, DotLottiePlayerWasm } = await import(
  new URL("release/wasm/dotlottie_rs.js", root)
);
initSync({ module: readFileSync(new URL("release/wasm/dotlottie_rs_bg.wasm", root)) });

const player = new DotLottiePlayerWasm();
player.setup_sw_target(WIDTH, HEIGHT);

const loaded = String(animationPath).endsWith(".lottie")
  ? player.load_dotlottie_data(readFileSync(animationPath))
  : player.load_animation(readFileSync(animationPath, "utf8"));
if (!loaded) {
  throw new Error(`Failed to load ${animationPath}`);
}

player.set_frame(Number(frame));
player.render();

writeFileSync(outputPath, bmp(WIDTH, HEIGHT, player.get_pixel_buffer()));
console.log(`Wrote frame ${frame} of ${player.total_frames()} to ${outputPath}`);

player.free();

function bmp(width, height, rgba) {
  const headerSize = 54;
  const out = Buffer.alloc(headerSize + rgba.length);

  out.write("BM", 0);
  out.writeUInt32LE(out.length, 2);
  out.writeUInt32LE(headerSize, 10);
  out.writeUInt32LE(40, 14); // DIB header size
  out.writeInt32LE(width, 18);
  out.writeInt32LE(-height, 22); // negative: rows run top to bottom
  out.writeUInt16LE(1, 26); // color planes
  out.writeUInt16LE(32, 28); // bits per pixel

  // BMP stores pixels as B, G, R, A.
  for (let i = 0; i < rgba.length; i += 4) {
    out[headerSize + i] = rgba[i + 2];
    out[headerSize + i + 1] = rgba[i + 1];
    out[headerSize + i + 2] = rgba[i];
    out[headerSize + i + 3] = rgba[i + 3];
  }

  return out;
}
