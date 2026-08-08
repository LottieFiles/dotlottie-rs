use js_sys::Array;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, BlobPropertyBag, CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, Url,
};

use super::MediaPlayer;

const DRIFT_TOLERANCE: f32 = 0.10;
const JUMP_THRESHOLD: f32 = 0.34;
const MAX_PLAYBACK_STEP: f32 = 0.20;
const KICK_INTERVAL: u32 = 30;

pub struct WebVideoPlayer {
    element: HtmlVideoElement,
    url: String,
    canvas: Option<(HtmlCanvasElement, CanvasRenderingContext2d)>,
    pixels: Vec<u8>,
    target: f32,
    step: f32,
    since_kick: u32,
}

impl WebVideoPlayer {
    pub fn open(data: &[u8]) -> Option<Self> {
        let array = Array::new();
        array.push(&js_sys::Uint8Array::from(data));

        let props = BlobPropertyBag::new();
        props.set_type("video/mp4");
        let blob = Blob::new_with_u8_array_sequence_and_options(&array, &props).ok()?;
        let url = Url::create_object_url_with_blob(&blob).ok()?;

        let document = web_sys::window().and_then(|w| w.document());
        let element: HtmlVideoElement = match document
            .as_ref()
            .and_then(|d| d.create_element("video").ok())
            .and_then(|e| e.dyn_into::<HtmlVideoElement>().ok())
        {
            Some(element) => element,
            None => {
                let _ = Url::revoke_object_url(&url);
                return None;
            }
        };
        element.set_src(&url);
        element.set_muted(true);
        element.set_attribute("playsinline", "").ok();
        element.set_preload("auto");

        if let Some(body) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            element
                .style()
                .set_css_text("position:fixed;left:-9999px;top:0;width:2px;height:2px;opacity:0");
            let _ = body.append_child(&element);
        }
        let _ = element.load();
        let _ = element.play();

        Some(Self {
            element,
            url,
            canvas: None,
            pixels: Vec::new(),
            target: 0.0,
            step: 0.0,
            since_kick: 0,
        })
    }

    fn canvas(&mut self, width: u32, height: u32) -> Option<CanvasRenderingContext2d> {
        if self.canvas.is_none() {
            let document = web_sys::window()?.document()?;
            let canvas: HtmlCanvasElement =
                document.create_element("canvas").ok()?.dyn_into().ok()?;
            canvas.set_width(width);
            canvas.set_height(height);
            let ctx: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
            self.canvas = Some((canvas, ctx));
        }
        self.canvas.as_ref().map(|(_, ctx)| ctx.clone())
    }

    fn track(&mut self) {
        if self.element.seeking() {
            return;
        }

        let drift = self.target - self.element.current_time() as f32;
        let playing_forward = self.step > 0.0 && self.step <= MAX_PLAYBACK_STEP;

        if drift.abs() > JUMP_THRESHOLD || !playing_forward {
            if !self.element.paused() {
                let _ = self.element.pause();
            }
            self.element.set_current_time(self.target as f64);
        } else {
            if self.element.paused() {
                let _ = self.element.play();
            }
            let rate = if drift.abs() > DRIFT_TOLERANCE {
                (1.0 + drift * 2.0).clamp(0.5, 1.5)
            } else {
                1.0
            };
            self.element.set_playback_rate(rate as f64);
        }
    }

    fn capture(&mut self, width: u32, height: u32, expected: usize) {
        let Some(ctx) = self.canvas(width, height) else {
            return;
        };
        if ctx
            .draw_image_with_html_video_element(&self.element, 0.0, 0.0)
            .is_err()
        {
            return;
        }
        let Ok(data) = ctx.get_image_data(0.0, 0.0, width as f64, height as f64) else {
            return;
        };
        let raw = data.data();
        if raw.len() < expected {
            return;
        }
        if self.pixels.len() != expected {
            self.pixels.resize(expected, 0);
        }
        self.pixels.copy_from_slice(&raw[..expected]);
    }
}

impl MediaPlayer for WebVideoPlayer {
    fn seek(&mut self, seconds: f32) {
        self.step = seconds - self.target;
        self.target = seconds;
    }

    fn info(&self) -> Option<(u32, u32, f32)> {
        let width = self.element.video_width();
        let height = self.element.video_height();
        let duration = self.element.duration() as f32;
        if width == 0 || height == 0 || !duration.is_finite() || duration <= 0.0 {
            return None;
        }
        Some((width, height, duration))
    }

    fn frame(&mut self) -> Option<&[u8]> {
        let (width, height, _) = self.info()?;

        if self.element.ready_state() < 2 {
            self.since_kick += 1;
            if self.since_kick >= KICK_INTERVAL {
                self.since_kick = 0;
                let _ = self.element.load();
                let _ = self.element.play();
            }
            return None;
        }

        self.track();

        let expected = (width as usize) * (height as usize) * 4;
        self.capture(width, height, expected);

        if self.pixels.len() == expected {
            Some(&self.pixels)
        } else {
            None
        }
    }

    fn set_playing(&mut self, _playing: bool) {}

    fn set_volume(&mut self, volume: f32) {
        self.element.set_volume(volume as f64);
    }

    fn set_muted(&mut self, muted: bool) {
        self.element.set_muted(muted);
    }

    fn time(&self) -> f32 {
        self.element.current_time() as f32
    }
}

impl Drop for WebVideoPlayer {
    fn drop(&mut self) {
        let _ = self.element.pause();
        self.element.remove();
        let _ = Url::revoke_object_url(&self.url);
    }
}
