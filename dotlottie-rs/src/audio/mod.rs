use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde_json::Value;

#[cfg(feature = "audio")]
mod rodio_player;
#[cfg(feature = "audio")]
pub use rodio_player::RodioPlayer;

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const DATA_AUDIO_PREFIX: &str = "data:audio/";

/// An audio asset decoded from a Lottie JSON file.
pub struct AudioAsset {
    pub id: String,
    pub data: Arc<[u8]>,
}

/// An audio layer extracted from a Lottie JSON file (`ty == 6`).
pub struct AudioLayer {
    pub ref_id: String,
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
/// Returns `(assets, layers)`.
pub fn extract_audio(json_data: &str) -> (Vec<AudioAsset>, Vec<AudioLayer>) {
    let root: Value = match serde_json::from_str(json_data) {
        Ok(v) => v,
        Err(_) => return (Vec::new(), Vec::new()),
    };

    let mut assets: Vec<AudioAsset> = Vec::new();

    if let Some(assets_arr) = root.get("assets").and_then(|v| v.as_array()) {
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

            assets.push(AudioAsset { id, data: Arc::from(decoded) });
        }
    }

    // Build a set of known audio asset IDs for fast lookup.
    let asset_ids: HashSet<&str> = assets.iter().map(|a| a.id.as_str()).collect();

    // --- Build a map: precomp_asset_id -> Vec<(audio_ref_id, volume)> ---
    // Handle audio layers inside precomps
    let mut audio_precomp_map: HashMap<&str, Vec<(String, f32)>> = HashMap::new();

    if let Some(assets_arr) = root.get("assets").and_then(|v| v.as_array()) {
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
                    Some(s) => s.to_string(),
                    None => continue,
                };

                if !asset_ids.contains(ref_id.as_str()) {
                    continue;
                }

                let volume = inner
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
                    .unwrap_or(1.0);

                audio_precomp_map
                    .entry(asset_id)
                    .or_default()
                    .push((ref_id, volume));
            }
        }
    }

    // --- Extract AudioLayers from the main (root) timeline only ---
    let mut layers: Vec<AudioLayer> = Vec::new();

    if let Some(root_layers) = root.get("layers").and_then(|v| v.as_array()) {
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

                    for (audio_ref_id, volume) in audio_infos {
                        layers.push(AudioLayer {
                            ref_id: audio_ref_id.clone(),
                            start_frame,
                            end_frame,
                            volume: *volume,
                            playing: false,
                        });
                    }
                }
                Some(6) => {
                    // Direct audio layer in the timeline (not wrapped in a precomp).
                    // Use its own ip/op directly.
                    let ref_id = match layer.get("refId").and_then(|v| v.as_str()) {
                        Some(s) => s.to_string(),
                        None => continue,
                    };

                    if !asset_ids.contains(ref_id.as_str()) {
                        continue;
                    }

                    let start_frame =
                        layer.get("ip").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let end_frame = layer.get("op").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

                    let volume = 1.0;

                    layers.push(AudioLayer {
                        ref_id,
                        start_frame,
                        end_frame,
                        volume,
                        playing: false,
                    });
                }
                _ => continue,
            }
        }
    }

    (assets, layers)
}

// ---------------------------------------------------------------------------
// AudioManager
// ---------------------------------------------------------------------------

/// Manages frame-synchronised audio playback.
///
/// Audio is played via rodio on native targets (macOS, iOS, Android) and on
/// `wasm32-unknown-unknown` (Web Audio API backend).
pub struct AudioManager {
    layers: Vec<AudioLayer>,
    /// Audio data keyed by asset id. Arc allows zero-copy hand-off to the player.
    assets: HashMap<String, Arc<[u8]>>,
    muted: bool,
    /// Global volume multiplier in [0.0, 1.0], applied on top of per-layer volume.
    volume: f32,
    rodio_player: RodioPlayer,
}

impl AudioManager {
    /// Returns `None` if there are no audio layers or if the audio backend fails to initialize.
    pub fn new(assets: Vec<AudioAsset>, layers: Vec<AudioLayer>) -> Option<Self> {
        if layers.is_empty() {
            return None;
        }

        let rodio_player = RodioPlayer::new(layers.len()).ok()?;
        let assets: HashMap<String, Arc<[u8]>> = assets
            .into_iter()
            .map(|a| (a.id, a.data))
            .collect();

        Some(AudioManager {
            layers,
            assets,
            muted: false,
            volume: 1.0,
            rodio_player,
        })
    }

    /// Synchronise audio state with the current animation frame.
    ///
    /// Returns events that the host application may handle.
    pub fn update(&mut self, frame: f32) {
        for (idx, layer) in self.layers.iter_mut().enumerate() {
            let should_play = frame >= layer.start_frame && frame < layer.end_frame;

            if should_play && !layer.playing {
                layer.playing = true;
                if let Some(data) = self.assets.get(&layer.ref_id) {
                    let vol = if self.muted { 0.0 } else { layer.volume * self.volume };
                    self.rodio_player.play(idx, data.clone(), vol);
                }
            } else if !should_play && layer.playing {
                layer.playing = false;
                self.rodio_player.stop(idx);
            }
        }
    }

    pub fn pause(&mut self) {
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.playing {
                self.rodio_player.pause(idx);
            }
        }
    }

    pub fn play(&mut self) {
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.playing {
                self.rodio_player.resume(idx);
            }
        }
    }

    pub fn stop(&mut self) {
        for (idx, layer) in self.layers.iter_mut().enumerate() {
            if layer.playing {
                self.rodio_player.stop(idx);
                layer.playing = false;
            }
        }
    }

    pub fn set_mute(&mut self, muted: bool) {
        self.muted = muted;
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.playing {
                let vol = if muted { 0.0 } else { layer.volume * self.volume };
                self.rodio_player.set_volume(idx, vol);
            }
        }
    }

    /// Set the global volume multiplier (clamped to [0.0, 1.0]).
    /// Applied on top of per-layer volume. Takes effect immediately for any
    /// currently-playing audio.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.playing {
                let vol = layer.volume * self.volume;
                self.rodio_player.set_volume(idx, vol);
            }
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}
