#![cfg(feature = "state-machines")]

// Spike: control a PRECOMP's timeline with a slot.
//
// Mechanism (precomp_scrub.json): the precomp layer's time remap (`tm`)
// carries an authored expression reading a hidden driver layer's ROTATION
// (a slot-supported scalar). Writing the "inner_time" scalar slot therefore
// scrubs the precomp's internal clock, renderer-side:
//
//   slot inner_time (seconds) -> driver rotation -> tm expression
//     -> precomp internal frame = seconds * fr
//
// NOTE: a `sid` directly on `tm` is NOT usable — ThorVG registers it against
// a null object and would crash on apply. The driver-layer bridge is the
// safe pattern.
//
// The precomp contains a red box gliding x:100 -> 400 over its 150 frames.
// The MAIN timeline is held at frame 0 the whole time (autoplay off), so
// any box movement is purely the slot-driven scrub. Verified by sampling
// rendered pixels.

#[cfg(test)]
mod tests {
    use dotlottie_rs::{ColorSpace, Player, ScalarSlot};
    use std::ffi::CString;

    fn red_at(buffer: &[u32], x: usize, y: usize) -> bool {
        // ABGR8888: red pixels have a high R byte and low G/B bytes.
        let px = buffer[y * 512 + x];
        let r = px & 0xFF;
        let g = (px >> 8) & 0xFF;
        let b = (px >> 16) & 0xFF;
        r > 200 && g < 60 && b < 60
    }

    #[test]
    fn slot_scrubs_precomp_timeline() {
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/precomp_scrub.json").unwrap();
        player.load_animation_path(&path).unwrap();

        // Drive like a real host: play + tick. The MAIN timeline may
        // advance freely — every main-comp layer is static, and the
        // precomp's internal clock is owned entirely by the tm expression.
        let _ = player.play();
        let _ = player.tick(16.0);
        assert!(
            red_at(&buffer, 100, 256),
            "precomp at internal frame 0: box at x=100"
        );
        assert!(!red_at(&buffer, 250, 256), "box not yet at midpoint");

        // Scrub: 2.5 seconds -> internal frame 75 -> box at x=250.
        player
            .set_scalar_slot("inner_time", ScalarSlot::new(2.5))
            .unwrap();
        let _ = player.tick(16.0);
        assert!(
            red_at(&buffer, 250, 256),
            "slot=2.5s should scrub the box to x=250"
        );
        assert!(!red_at(&buffer, 100, 256), "box should have left x=100");

        // Scrub near the end: 4.9s -> internal frame 147 -> box at x~394.
        // (Exactly 5.0s maps to frame 150 == the layer's op, which is
        // exclusive — the layer is not rendered at its very end frame.)
        player
            .set_scalar_slot("inner_time", ScalarSlot::new(4.9))
            .unwrap();
        let _ = player.tick(16.0);
        assert!(
            red_at(&buffer, 394, 256),
            "slot=4.9s should scrub the box to x~394"
        );
        assert!(
            !red_at(&buffer, 250, 256),
            "box should have left the midpoint"
        );
    }
}
