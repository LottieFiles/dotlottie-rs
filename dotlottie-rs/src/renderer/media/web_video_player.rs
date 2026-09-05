use js_sys::Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Blob, BlobPropertyBag, CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, Url,
};

thread_local! {
    /// Reused no-op rejection handler for [`play`]; built once so the per-frame
    /// calls do not allocate a closure each time.
    static IGNORE_REJECTION: Closure<dyn FnMut(JsValue)> = Closure::new(|_| {});
}

/// Start playback, absorbing the promise rejection.
///
/// `HTMLMediaElement.play()` rejects routinely here — the browser blocks
/// autoplay, or a `load()`/seek interrupts the pending request, since the
/// element is driven from the animation frame rather than by the user. Dropping
/// the promise would surface those as unhandled rejections in the host page.
fn play(element: &HtmlVideoElement) {
    let Ok(promise) = element.play() else {
        return;
    };
    IGNORE_REJECTION.with(|noop| {
        let _ = promise.catch(noop);
    });
}

const KICK_INTERVAL: u32 = 30;
/// Chrome's supported `playbackRate` range.
const MIN_RATE: f64 = 0.0625;
const MAX_RATE: f64 = 16.0;

/// A hidden `<video>` behind ThorVG's media loader.
///
/// ThorVG lets the clip free-run and only seeks it when it drifts more than a
/// second from the animation clock, so the element's own clock carries
/// playback. It plays only while ThorVG wants the layer playing *and* the
/// dotLottie player is not paused, at the player's speed; it cannot play
/// backwards, so reverse playback holds the frame between ThorVG's seeks.
pub struct WebVideoPlayer {
    element: HtmlVideoElement,
    url: String,
    canvas: Option<(HtmlCanvasElement, CanvasRenderingContext2d)>,
    pixels: Vec<u8>,
    /// ThorVG's `play`/`pause`/`stop` for the layer.
    wanted: bool,
    /// The dotLottie player is paused or stopped.
    halted: bool,
    /// Signed speed; negative means the animation runs backwards.
    rate: f32,
    since_kick: u32,
}

impl WebVideoPlayer {
    pub fn open(data: &[u8], halted: bool, rate: f32) -> Option<Self> {
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

        if let Some(body) = document.and_then(|d| d.body()) {
            element
                .style()
                .set_css_text("position:fixed;left:-9999px;top:0;width:2px;height:2px;opacity:0");
            let _ = body.append_child(&element);
        }
        // `preload` is only a hint; a muted play() makes the browser fetch and
        // decode. `apply` pauses it again once frames are available if nothing
        // wants it playing.
        let _ = element.load();
        play(&element);

        Some(Self {
            element,
            url,
            canvas: None,
            pixels: Vec::new(),
            wanted: false,
            halted,
            rate,
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

    /// Bring the element's play state and rate in line with what is wanted.
    fn apply(&mut self) {
        if self.element.ready_state() < 2 {
            return;
        }
        if self.wanted && !self.halted && self.rate > 0.0 {
            let rate = (self.rate as f64).clamp(MIN_RATE, MAX_RATE);
            if (self.element.playback_rate() - rate).abs() > f64::EPSILON {
                self.element.set_playback_rate(rate);
            }
            if self.element.paused() {
                play(&self.element);
            }
        } else if !self.element.paused() {
            let _ = self.element.pause();
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

impl WebVideoPlayer {
    pub fn seek(&mut self, seconds: f32) {
        self.element.set_current_time(seconds as f64);
    }

    pub fn info(&self) -> Option<(u32, u32, f32)> {
        let width = self.element.video_width();
        let height = self.element.video_height();
        let duration = self.element.duration() as f32;
        if width == 0 || height == 0 || !duration.is_finite() || duration <= 0.0 {
            return None;
        }
        Some((width, height, duration))
    }

    pub fn frame(&mut self) -> Option<&[u8]> {
        let (width, height, _) = self.info()?;

        if self.element.ready_state() < 2 {
            self.since_kick += 1;
            if self.since_kick >= KICK_INTERVAL {
                self.since_kick = 0;
                let _ = self.element.load();
                play(&self.element);
            }
            return None;
        }

        self.apply();

        let expected = (width as usize) * (height as usize) * 4;
        self.capture(width, height, expected);

        if self.pixels.len() == expected {
            Some(&self.pixels)
        } else {
            None
        }
    }

    /// ThorVG wants the layer playing or not.
    pub fn set_layer_playing(&mut self, playing: bool) {
        self.wanted = playing;
        self.apply();
    }

    /// The dotLottie player resumed or paused.
    pub fn set_playing(&mut self, playing: bool) {
        self.halted = !playing;
        self.apply();
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
        self.apply();
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.element.set_volume(volume as f64);
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.element.set_muted(muted);
    }

    pub fn time(&self) -> f32 {
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
