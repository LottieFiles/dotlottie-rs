//! Runtime for the DragAndDrop interaction's PATH MODE (prototype) —
//! selected by `pathLayerName` on the interaction.
//!
//! A path-constrained drag "sensor": while held, the pointer is projected
//! onto the authored bezier path of a named layer, and the normalized
//! arc-length progress (plus optionally the projected point) is written
//! into Numeric inputs. Everything visual happens downstream through the
//! existing machinery — live-bound state slots, guards, or a time-remapped
//! precomp driven by the progress.
//!
//! Projection uses a flattened arc-length sample table built once at
//! start(): each cubic segment is sampled uniformly, and pointer projection
//! is a nearest-sample scan (the table is small; a few hundred entries).

use crate::lottie_renderer::slots::LayerPath;

use super::actions::Action;
use super::drag_and_drop::DndZone;
use super::interactions::Interaction;

const SAMPLES_PER_SEGMENT: usize = 24;

#[derive(Debug, Clone)]
pub(crate) struct PathSample {
    pub point: [f32; 2],
    /// Cumulative arc length at this sample.
    pub len: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PathSnap {
    pub from: f32,
    pub to: f32,
    pub elapsed: f32,
    pub duration: f32,
    pub easing: [f32; 4],
}

#[derive(Debug, Clone)]
pub(crate) struct PathDragRuntime {
    pub layer_name: String,
    pub path_layer_name: String,
    pub progress_input: String,
    pub state_name: Option<String>,
    pub on_grab: Vec<Action>,
    pub on_drag: Vec<Action>,
    pub on_drop: Vec<Action>,
    /// Drop zones as DOCK POINTS ON THE PATH: at release, the zone under
    /// the pointer snaps `progress_input` to the zone's own on-path
    /// position and runs its actions. `snap`/`track` are ignored in path
    /// mode (progress is the only output).
    pub zones: Vec<DndZone>,
    /// Uncaptured-release fallback: "previous" ratchets back to the
    /// nearest zone behind the release progress; "nearest" goes either
    /// direction; None = no fallback.
    pub dock_fallback: Option<String>,
    /// (duration ms, easing) for the dock glide; None = snap instantly.
    pub tween: Option<(f32, [f32; 4])>,
    /// In-flight dock glide in PROGRESS space: the engine animates the
    /// progress input from release to the zone's position, running onDrag
    /// each tick so the object slides ALONG THE PATH into the dock.
    pub snapping: Option<PathSnap>,
    pub locked: bool,
    pub held: bool,
    pub samples: Vec<PathSample>,
    pub total_len: f32,
    /// Current position along the path as a sample index — the drag's
    /// single source of truth for branch locality (spirals/overlaps).
    pub current_index: usize,
    /// Last pointer position, for speed-adaptive search windows.
    pub last_pointer: Option<[f32; 2]>,
}

/// Window tuning, relative to the path's total arc length.
const BASE_WINDOW_FRAC: f32 = 0.04;
const MAX_WINDOW_FRAC: f32 = 0.10;
const WINDOW_GROWTH: f32 = 1.5;
const MAX_CHASE_ITERATIONS: usize = 8;

fn cubic(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0],
        a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1],
    ]
}

impl PathDragRuntime {
    pub(crate) fn from_interaction(interaction: &Interaction) -> Option<Self> {
        let Interaction::DragAndDrop {
            layer_name,
            path_layer_name,
            progress_input,
            dock_fallback,
            state_name,
            tween,
            on_grab,
            on_drag,
            on_drop,
            drop_zones,
            ..
        } = interaction
        else {
            return None;
        };

        // Path mode only: needs both the constraint curve and a progress
        // output. (Free/bounded drags route to DndRuntime.)
        let path_layer_name = path_layer_name.as_ref()?;
        let progress_input = progress_input.as_ref()?;

        Some(PathDragRuntime {
            layer_name: layer_name.as_str().to_owned(),
            path_layer_name: path_layer_name.as_str().to_owned(),
            progress_input: progress_input.as_str().to_owned(),
            state_name: state_name.as_ref().map(|s| s.as_str().to_owned()),
            on_grab: on_grab.clone().unwrap_or_default(),
            on_drag: on_drag.clone().unwrap_or_default(),
            on_drop: on_drop.clone().unwrap_or_default(),
            zones: drop_zones
                .iter()
                .map(|z| DndZone {
                    layer_name: z.layer_name.as_str().to_owned(),
                    snap: z.snap,
                    lock: z.lock.unwrap_or(false),
                    track: z.track.unwrap_or(false),
                    actions: z.actions.clone().unwrap_or_default(),
                })
                .collect(),
            dock_fallback: dock_fallback.as_ref().map(|s| s.as_str().to_owned()),
            // Seconds -> milliseconds, same convention as free mode.
            tween: tween.as_ref().map(|t| (t.duration * 1000.0, t.easing)),
            snapping: None,
            locked: false,
            held: false,
            samples: Vec::new(),
            total_len: 0.0,
            current_index: 0,
            last_pointer: None,
        })
    }

    /// Flatten the bezier into an arc-length sample table.
    pub(crate) fn build_samples(&mut self, path: &LayerPath) {
        self.samples.clear();
        self.total_len = 0.0;

        let n = path.verts.len();
        if n < 2 {
            return;
        }

        let segment_count = if path.closed { n } else { n - 1 };
        let mut prev: Option<[f32; 2]> = None;

        for seg in 0..segment_count {
            let k0 = seg;
            let k1 = (seg + 1) % n;
            let p0 = path.verts[k0];
            let p3 = path.verts[k1];
            let p1 = [
                p0[0] + path.out_tangents[k0][0],
                p0[1] + path.out_tangents[k0][1],
            ];
            let p2 = [
                p3[0] + path.in_tangents[k1][0],
                p3[1] + path.in_tangents[k1][1],
            ];

            let start = if seg == 0 { 0 } else { 1 };
            for i in start..=SAMPLES_PER_SEGMENT {
                let t = i as f32 / SAMPLES_PER_SEGMENT as f32;
                let point = cubic(p0, p1, p2, p3, t);
                if let Some(prev) = prev {
                    self.total_len +=
                        ((point[0] - prev[0]).powi(2) + (point[1] - prev[1]).powi(2)).sqrt();
                }
                self.samples.push(PathSample {
                    point,
                    len: self.total_len,
                });
                prev = Some(point);
            }
        }
    }

    fn nearest_in(&self, pointer: [f32; 2], lo: usize, hi: usize) -> usize {
        let mut best = lo;
        let mut best_d2 = f32::MAX;
        for (idx, sample) in self.samples[lo..=hi].iter().enumerate() {
            let d2 =
                (sample.point[0] - pointer[0]).powi(2) + (sample.point[1] - pointer[1]).powi(2);
            if d2 < best_d2 {
                best_d2 = d2;
                best = lo + idx;
            }
        }
        best
    }

    /// Arc-length progress of the point's global-nearest sample. Used for
    /// dock points, which sit ON the path — branch locality is moot.
    pub(crate) fn progress_at_point(&self, point: [f32; 2]) -> Option<f32> {
        if self.samples.is_empty() || self.total_len <= 0.0 {
            return None;
        }
        let idx = self.nearest_in(point, 0, self.samples.len() - 1);
        Some(self.samples[idx].len / self.total_len)
    }

    /// Seed the drag at grab time: global nearest (the pointer is on the
    /// grabbed object, so the nearest sample IS the correct branch).
    pub(crate) fn seed(&mut self, pointer: [f32; 2]) {
        if self.samples.is_empty() {
            return;
        }
        self.current_index = self.nearest_in(pointer, 0, self.samples.len() - 1);
        self.last_pointer = Some(pointer);
    }

    /// Constrained projection: search only an arc-length window around the
    /// current index, growing with pointer speed and chasing along the
    /// path if the best match sits on the window edge. A fast pointer is
    /// followed along the path; a spatially-near-but-arc-far branch of a
    /// spiral is unreachable. Returns the projected point + progress.
    pub(crate) fn project_windowed(&mut self, pointer: [f32; 2]) -> Option<([f32; 2], f32)> {
        if self.samples.is_empty() || self.total_len <= 0.0 {
            return None;
        }

        let speed = self
            .last_pointer
            .map(|last| ((pointer[0] - last[0]).powi(2) + (pointer[1] - last[1]).powi(2)).sqrt())
            .unwrap_or(0.0);
        self.last_pointer = Some(pointer);

        let window_len = (self.total_len * BASE_WINDOW_FRAC + speed * WINDOW_GROWTH)
            .min(self.total_len * MAX_WINDOW_FRAC);

        let n = self.samples.len();
        let mut idx = self.current_index.min(n - 1);

        for _ in 0..MAX_CHASE_ITERATIONS {
            let center_len = self.samples[idx].len;
            let lo = self
                .samples
                .partition_point(|s| s.len < center_len - window_len);
            let hi = self
                .samples
                .partition_point(|s| s.len <= center_len + window_len)
                .saturating_sub(1)
                .min(n - 1);

            let best = self.nearest_in(pointer, lo, hi);
            let on_edge = (best == lo && lo > 0) || (best == hi && hi < n - 1);
            idx = best;
            if !on_edge {
                break;
            }
        }

        self.current_index = idx;
        let sample = &self.samples[idx];
        Some((sample.point, sample.len / self.total_len))
    }
}
