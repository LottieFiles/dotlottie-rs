// Fails the build if the wasm module imports anything from the `env` module.
//
// On wasm32-unknown-unknown there is no libc/GL/WebGPU runtime to link against, so any
// C symbol ThorVG references that we forgot to define in src/wasm/*_stubs.rs is silently
// turned into an import from `env`. wasm-bindgen then emits `import * as __wbg_star0 from 'env'`,
// which bundlers cannot resolve and which throws a LinkError at instantiation.

const fs = require("fs");

const wasmPath = process.argv[2];

if (!wasmPath) {
  console.error("usage: node scripts/check_wasm_imports.js <path-to-wasm>");
  process.exit(2);
}

const module_ = new WebAssembly.Module(fs.readFileSync(wasmPath));
const missing = WebAssembly.Module.imports(module_).filter((i) => i.module === "env");

if (missing.length > 0) {
  console.error(`\nerror: ${wasmPath} has ${missing.length} undefined symbol(s) imported from 'env':\n`);
  for (const { kind, name } of missing) {
    console.error(`    ${kind} ${name}`);
  }
  console.error("\nAdd a #[no_mangle] stub for each in dotlottie-rs/src/wasm/.\n");
  process.exit(1);
}
