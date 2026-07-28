// Shared boilerplate for the paint-fx use-case examples.
// Each page: const { player, onFrame } = await initPlayer(canvas, "assets/x.json");

let mod = null;

export async function loadWasm() {
  if (mod) return mod;
  mod = await import("../../release/wasm/dotlottie_rs.js");
  await mod.default({ module_or_path: "../../release/wasm/dotlottie_rs_bg.wasm" });
  return mod;
}

const players = [];
if (typeof window !== "undefined") window.__fxPlayers = players;

export async function initPlayer(canvas, assetUrl, { size = 512, loop = true } = {}) {
  const m = await loadWasm();
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext("2d");
  const player = new m.DotLottiePlayerWasm();
  if (!player.setup_sw_target(size, size)) throw new Error("setup_sw_target failed");
  const data = await fetch(assetUrl).then((r) => r.text());
  if (!player.load_animation(data)) throw new Error("load_animation failed: " + assetUrl);
  player.set_loop(loop);
  player.play();

  const entry = { player, canvas, ctx, size, update: null };
  players.push(entry);
  startLoop();
  return entry;
}

let rafStarted = false;
function startLoop() {
  if (rafStarted) return;
  rafStarted = true;
  let last = 0;
  function frame(now) {
    requestAnimationFrame(frame);
    const dtMs = last ? now - last : 16;
    const dt = Math.min(0.05, dtMs / 1000);
    last = now;
    for (const e of players) {
      e.update?.(dt, now);
      let drew = e.player.tick(dtMs);
      if (!drew) drew = e.player.render();
      if (drew) {
        const px = e.player.get_pixel_buffer();
        const clamped = new Uint8ClampedArray(px.buffer, px.byteOffset, px.byteLength);
        e.ctx.clearRect(0, 0, e.size, e.size);
        e.ctx.putImageData(new ImageData(clamped, e.size, e.size), 0, 0);
      }
    }
  }
  requestAnimationFrame(frame);
}

// ── SVG path data → flat Tvg_Path_Command buffers ──────────────────────
// Supports M/L/C/Z plus H/V/S (absolute and relative), i.e. what design
// tools export for untransformed paths. Returns { cmds: Uint8Array,
// pts: Float32Array } ready for set_clip_path / set_overlay_path.
export function parseSvgPath(d, scale = 1) {
  const CLOSE = 0, MOVE = 1, LINE = 2, CUBIC = 3;
  const cmds = [], pts = [];
  const tokens = d.match(/[a-df-z]|-?(?:\d*\.\d+|\d+)(?:e[-+]?\d+)?/gi) ?? [];
  let i = 0, x = 0, y = 0, sx = 0, sy = 0, kx = 0, ky = 0, cmd = "", fresh = false;
  const num = () => parseFloat(tokens[i++]);
  const put = (px, py) => pts.push(px * scale, py * scale);
  while (i < tokens.length) {
    if (/[a-z]/i.test(tokens[i])) { cmd = tokens[i++]; fresh = true; }
    const rel = cmd === cmd.toLowerCase();
    const op = cmd.toUpperCase();
    if (op === "Z") {
      cmds.push(CLOSE);
      x = sx; y = sy;
      fresh = false;
      continue;
    }
    if (op === "M" || op === "L") {
      const nx = num() + (rel ? x : 0), ny = num() + (rel ? y : 0);
      // Extra M coordinate pairs are implicit LineTos.
      const isMove = op === "M" && fresh;
      cmds.push(isMove ? MOVE : LINE);
      if (isMove) { sx = nx; sy = ny; }
      x = nx; y = ny;
      put(x, y);
      kx = x; ky = y;
    } else if (op === "H" || op === "V") {
      if (op === "H") x = num() + (rel ? x : 0);
      else y = num() + (rel ? y : 0);
      cmds.push(LINE);
      put(x, y);
      kx = x; ky = y;
    } else if (op === "C" || op === "S") {
      let c1x, c1y;
      if (op === "C") {
        c1x = num() + (rel ? x : 0); c1y = num() + (rel ? y : 0);
      } else {
        c1x = 2 * x - kx; c1y = 2 * y - ky; // reflect previous control
      }
      const c2x = num() + (rel ? x : 0), c2y = num() + (rel ? y : 0);
      const nx = num() + (rel ? x : 0), ny = num() + (rel ? y : 0);
      cmds.push(CUBIC);
      put(c1x, c1y); put(c2x, c2y); put(nx, ny);
      kx = c2x; ky = c2y;
      x = nx; y = ny;
    } else {
      throw new Error(`parseSvgPath: unsupported command ${cmd}`);
    }
    fresh = false;
  }
  return { cmds: new Uint8Array(cmds), pts: new Float32Array(pts) };
}

// ── 3x3 row-major matrix helpers ───────────────────────────────────────
export const I = () => [1, 0, 0, 0, 1, 0, 0, 0, 1];
export const T = (x, y) => [1, 0, x, 0, 1, y, 0, 0, 1];
export const S = (sx, sy) => [sx, 0, 0, 0, sy, 0, 0, 0, 1];
export const R = (deg) => {
  const r = (deg * Math.PI) / 180, c = Math.cos(r), s = Math.sin(r);
  return [c, -s, 0, s, c, 0, 0, 0, 1];
};
export function mul(a, b) {
  const o = new Array(9);
  for (let r = 0; r < 3; r++)
    for (let c = 0; c < 3; c++)
      o[r * 3 + c] = a[r * 3] * b[c] + a[r * 3 + 1] * b[3 + c] + a[r * 3 + 2] * b[6 + c];
  return o;
}
export const around = (cx, cy, m) => mul(T(cx, cy), mul(m, T(-cx, -cy)));
export const setT = (p, m) => p.set_transform(new Float32Array(m));
export const setLayerT = (p, name, m) => p.set_layer_transform(name, new Float32Array(m));

// Critically-damped-ish spring. Returns [value, velocity].
export function spring(cur, vel, target, dt, k = 180, d = 16) {
  const a = -k * (cur - target) - d * vel;
  vel += a * dt;
  cur += vel * dt;
  return [cur, vel];
}

// Pointer position in canvas pixel coords.
export function trackPointer(canvas, size, onMove, onLeave) {
  canvas.addEventListener("pointermove", (e) => {
    const r = canvas.getBoundingClientRect();
    onMove(((e.clientX - r.left) / r.width) * size, ((e.clientY - r.top) / r.height) * size, e);
  });
  canvas.addEventListener("pointerleave", () => onLeave?.());
}
