use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
mod rodio_player;
#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
pub use rodio_player::RodioPlayer;

#[cfg(all(feature = "audio", target_arch = "wasm32"))]
mod web_audio_player;
#[cfg(all(feature = "audio", target_arch = "wasm32"))]
use web_audio_player::WebAudioPlayer;

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const DATA_AUDIO_PREFIX: &str = "data:audio/";

/// An audio layer extracted from a Lottie JSON file (`ty == 6`),
/// with its asset already resolved to a direct index.
#[derive(Clone)]
pub struct AudioLayer {
    /// Index into the asset `Vec` held by `AudioManager`.
    pub asset_idx: usize,
    /// In-point frame (inclusive).
    pub start_frame: f32,
    /// Out-point frame (exclusive).
    pub end_frame: f32,
    /// Normalized volume (0.0–1.0).
    pub volume: f32,
    /// Whether this layer's audio is currently active (started but not stopped).
    pub playing: bool,
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    // Build reverse lookup table.
    let mut lookup = [0xFFu8; 256];
    for (i, &c) in BASE64_CHARS.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }

    // Strip padding characters.
    let input = input.trim_end_matches('=');
    let input_bytes = input.as_bytes();

    // Validate all characters are in the base64 alphabet.
    for &b in input_bytes {
        if lookup[b as usize] == 0xFF {
            return None;
        }
    }

    let output_len = (input_bytes.len() * 3) / 4;
    let mut output = Vec::with_capacity(output_len);

    let mut i = 0;
    while i + 3 < input_bytes.len() {
        let b0 = lookup[input_bytes[i] as usize] as u32;
        let b1 = lookup[input_bytes[i + 1] as usize] as u32;
        let b2 = lookup[input_bytes[i + 2] as usize] as u32;
        let b3 = lookup[input_bytes[i + 3] as usize] as u32;
        let n = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;
        output.push((n >> 16) as u8);
        output.push((n >> 8) as u8);
        output.push(n as u8);
        i += 4;
    }

    // Handle the last 2 or 3 input characters (1 or 2 output bytes).
    let remaining = input_bytes.len() - i;
    if remaining == 2 {
        let b0 = lookup[input_bytes[i] as usize] as u32;
        let b1 = lookup[input_bytes[i + 1] as usize] as u32;
        let n = (b0 << 18) | (b1 << 12);
        output.push((n >> 16) as u8);
    } else if remaining == 3 {
        let b0 = lookup[input_bytes[i] as usize] as u32;
        let b1 = lookup[input_bytes[i + 1] as usize] as u32;
        let b2 = lookup[input_bytes[i + 2] as usize] as u32;
        let n = (b0 << 18) | (b1 << 12) | (b2 << 6);
        output.push((n >> 16) as u8);
        output.push((n >> 8) as u8);
    }

    Some(output)
}

// ---------------------------------------------------------------------------
// JSON parsing
// ---------------------------------------------------------------------------

/// Parse audio assets and layers from a Lottie JSON string.
///
/// Returns `(assets, layers)` where each layer's `asset_idx` is already
/// resolved to its position in the returned `assets` Vec.
pub fn extract_audio(json_data: &Value) -> (Vec<Arc<[u8]>>, Vec<AudioLayer>) {
    // --- Pass 1: collect audio assets and build id → index map ---
    let mut raw_assets: Vec<(String, Arc<[u8]>)> = Vec::new();

    if let Some(assets_arr) = json_data.get("assets").and_then(|v| v.as_array()) {
        for asset in assets_arr {
            let id = match asset.get("id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let p = match asset.get("p").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };

            if !p.starts_with(DATA_AUDIO_PREFIX) {
                continue;
            }

            let after_prefix = &p[DATA_AUDIO_PREFIX.len()..];
            let semi_pos = match after_prefix.find(';') {
                Some(pos) => pos,
                None => continue,
            };

            let rest = &after_prefix[semi_pos + 1..];
            let b64_data = match rest.strip_prefix("base64,") {
                Some(d) => d,
                None => continue,
            };

            let decoded = match decode_base64(b64_data) {
                Some(d) => d,
                None => continue,
            };

            raw_assets.push((id, Arc::from(decoded)));
        }
    }

    let id_to_idx: HashMap<&str, usize> = raw_assets
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (id.as_str(), i))
        .collect();

    // --- Pass 2: build precomp map storing resolved asset indices ---
    let mut audio_precomp_map: HashMap<&str, Vec<(usize, f32)>> = HashMap::new();

    if let Some(assets_arr) = json_data.get("assets").and_then(|v| v.as_array()) {
        for asset in assets_arr {
            let asset_id = match asset.get("id").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };

            let inner_layers = match asset.get("layers").and_then(|v| v.as_array()) {
                Some(l) => l,
                None => continue,
            };

            for inner in inner_layers {
                if inner.get("ty").and_then(|v| v.as_u64()) != Some(6) {
                    continue;
                }

                let ref_id = match inner.get("refId").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => continue,
                };

                let asset_idx = match id_to_idx.get(ref_id) {
                    Some(&i) => i,
                    None => continue,
                };

                let volume = parse_volume(inner);

                audio_precomp_map
                    .entry(asset_id)
                    .or_default()
                    .push((asset_idx, volume));
            }
        }
    }

    // --- Pass 3: extract AudioLayers from the main (root) timeline ---
    let mut layers: Vec<AudioLayer> = Vec::new();

    if let Some(root_layers) = json_data.get("layers").and_then(|v| v.as_array()) {
        for layer in root_layers {
            match layer.get("ty").and_then(|v| v.as_u64()) {
                Some(0) => {
                    // Precomp instance layer — check if it wraps an audio precomp.
                    // The ip/op here are in the parent timeline and give us the
                    // correct playback window for the audio.
                    let ref_id = match layer.get("refId").and_then(|v| v.as_str()) {
                        Some(s) => s,
                        None => continue,
                    };

                    let audio_infos = match audio_precomp_map.get(ref_id) {
                        Some(infos) => infos,
                        None => continue,
                    };

                    let start_frame =
                        layer.get("ip").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let end_frame = layer.get("op").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

                    for &(asset_idx, volume) in audio_infos {
                        layers.push(AudioLayer {
                            asset_idx,
                            start_frame,
                            end_frame,
                            volume,
                            playing: false,
                        });
                    }
                }
                Some(6) => {
                    // Direct audio layer in the timeline (not wrapped in a precomp).
                    let ref_id = match layer.get("refId").and_then(|v| v.as_str()) {
                        Some(s) => s,
                        None => continue,
                    };

                    let asset_idx = match id_to_idx.get(ref_id) {
                        Some(&i) => i,
                        None => continue,
                    };

                    let start_frame =
                        layer.get("ip").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let end_frame = layer.get("op").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

                    layers.push(AudioLayer {
                        asset_idx,
                        start_frame,
                        end_frame,
                        volume: parse_volume(layer),
                        playing: false,
                    });
                }
                _ => continue,
            }
        }
    }

    let assets = raw_assets.into_iter().map(|(_, data)| data).collect();
    (assets, layers)
}

fn parse_volume(layer: &Value) -> f32 {
    layer
        .get("au")
        .and_then(|au| au.get("lv"))
        .and_then(|lv| lv.get("k"))
        .and_then(|k| {
            if let Some(arr) = k.as_array() {
                arr.first()
                    .and_then(|v| v.as_f64())
                    .map(|v| (v / 100.0) as f32)
            } else {
                k.as_f64().map(|v| (v / 100.0) as f32)
            }
        })
        .unwrap_or(1.0)
}

// ---------------------------------------------------------------------------
// AudioManager
// ---------------------------------------------------------------------------

/// Manages frame-synchronised audio playback.
///
/// On native targets (macOS, iOS, Android) audio is played via
/// rodio. On `wasm32-unknown-unknown` it delegates to the browser's native
/// `HtmlAudioElement` so no audio decoder needs to be bundled in the wasm binary.
pub struct AudioManager {
    layers: Vec<AudioLayer>,
    /// Audio data indexed by asset position; Arc allows zero-copy hand-off to the player.
    assets: Vec<Arc<[u8]>>,
    /// Global volume multiplier in [0.0, 1.0], applied on top of per-layer volume.
    volume: f32,
    #[cfg(not(target_arch = "wasm32"))]
    player: RodioPlayer,
    #[cfg(target_arch = "wasm32")]
    player: WebAudioPlayer,
}

impl AudioManager {
    /// Returns `None` if there are no audio layers or if the audio backend fails to initialize.
    pub fn with_assets(assets: Vec<Arc<[u8]>>, layers: Vec<AudioLayer>) -> Option<Self> {
        if layers.is_empty() {
            return None;
        }

        #[cfg(not(target_arch = "wasm32"))]
        let player = RodioPlayer::new(layers.len()).ok()?;
        #[cfg(target_arch = "wasm32")]
        let player = WebAudioPlayer::new(layers.len()).ok()?;

        Some(AudioManager {
            layers,
            assets,
            volume: 1.0,
            player,
        })
    }

    /// Synchronise audio state with the current animation frame.
    pub fn update(&mut self, frame: f32) {
        for (idx, layer) in self.layers.iter_mut().enumerate() {
            let should_play = frame >= layer.start_frame && frame < layer.end_frame;

            if should_play && !layer.playing {
                layer.playing = true;
                let data = &self.assets[layer.asset_idx];
                self.player.play(idx, data.clone());
                self.player.set_volume(idx, layer.volume * self.volume);
            } else if !should_play && layer.playing {
                layer.playing = false;
                self.player.stop(idx);
            }
        }
    }

    pub fn pause(&mut self) {
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.playing {
                self.player.pause(idx);
            }
        }
    }

    pub fn play(&mut self) {
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.playing {
                self.player.resume(idx);
            }
        }
    }

    pub fn stop(&mut self) {
        for (idx, layer) in self.layers.iter_mut().enumerate() {
            if layer.playing {
                self.player.stop(idx);
                layer.playing = false;
            }
        }
    }

    /// Set the global volume multiplier (clamped to [0.0, 1.0]).
    /// Applied on top of per-layer volume. Takes effect immediately for any
    /// currently-playing audio.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        for (idx, layer) in self.layers.iter().enumerate() {
            let vol = layer.volume * self.volume;
            self.player.set_volume(idx, vol);
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}
