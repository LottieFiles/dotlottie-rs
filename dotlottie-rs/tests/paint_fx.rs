use dotlottie_rs::{ColorSpace, Player};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: loading renders internally (load_animation_common), so `render()`
    // right after load correctly errors with "nothing to render". Assert on
    // buffer contents instead. Frame 21 is mid-animation, where the shape
    // layers are visible (frame 0 is a solid white fill, useless for pixel
    // comparisons).
    fn loaded_player(buffer: &mut [u32]) -> Player {
        let mut player = Player::new();
        assert!(player
            .set_sw_target(buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());
        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        let distinct = buffer.iter().collect::<std::collections::HashSet<_>>();
        assert!(distinct.len() > 1, "frame 21 should have visual variety");
        player
    }

    #[test]
    fn opacity_and_effects_change_pixels() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();
        assert!(
            baseline.iter().any(|&px| px != 0),
            "load's internal render drew nothing through the scene wrap"
        );

        assert!(player.set_opacity(60).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.set_opacity(255).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        assert!(player.add_gaussian_blur(8.0, 0, 0, 100).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.clear_effects().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // The white solid fills the whole canvas, hiding a drop shadow cast
        // behind it — shrink the content first so the shadow lands on empty
        // canvas.
        let shrink = vec![0.6, 0.0, 20.0, 0.0, 0.6, 20.0, 0.0, 0.0, 1.0];
        assert!(player.set_transform(shrink).is_ok());
        assert!(player.render().is_ok());
        let shrunk = buffer.clone();
        assert_ne!(shrunk, baseline);

        assert!(player.add_drop_shadow(0, 0, 0, 200, 45.0, 12.0, 6.0, 100).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, shrunk);

        assert!(player.clear_effects().is_ok());
        assert!(player
            .add_tritone(20, 20, 20, 128, 128, 128, 235, 235, 235, 0)
            .is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, shrunk);

        assert!(player.set_blend_mode(1).is_ok());
        assert!(player.render().is_ok());
    }

    #[test]
    fn layer_props_compose_and_restore() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();
        assert!(baseline.iter().any(|&px| px != 0));

        // Clean frame-22 reference for the survival assertion below.
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        let clean_frame22 = buffer.clone();
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        let shift = vec![1.0, 0.0, 200.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        assert!(player.set_layer_transform("R", shift).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.set_layer_opacity("E", 128).is_ok());
        assert!(player.render().is_ok());

        // Re-apply must survive a frame change (ThorVG rebuilds layer paints):
        // frame 22 with props active must differ from the clean frame 22.
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, clean_frame22);
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());

        assert!(player.clear_layer_props("R").is_ok());
        assert!(player.clear_layer_props("E").is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // Restore entries are dropped after one flush; playback then renders
        // identically to a prop-free player.
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, clean_frame22);

        // Unknown layer name is a no-op, not an error.
        assert!(player.set_layer_opacity("no-such-layer", 0).is_ok());
        assert!(player.render().is_ok());
    }

    #[test]
    fn layer_blur_and_visibility() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();

        // NOTE: per-layer sigma is in composition units (scaled by the layout
        // transform), unlike whole-scene blur which is in canvas pixels.
        // test.json is a 1500-unit comp on a 100px canvas → scale ≈ 0.067.
        assert!(player.set_layer_blur("R", 90.0, 100).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        // Survives frame changes (fresh scene, blur re-attached).
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        // Removing blur on a paused (non-rebuilt) scene detaches it.
        assert!(player.set_layer_blur("R", 0.0, 0).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        assert!(player.set_layer_visible("E", false).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.set_layer_visible("E", true).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
    }

    #[test]
    fn null_reference_layer_is_queryable() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = Player::new();
        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());
        let data = std::fs::read_to_string("../examples/web/assets/control.json").unwrap();
        let c_data = CString::new(data).unwrap();
        assert!(player.load_animation_data(&c_data).is_ok());

        // The ty:3 null layer renders nothing but exists in the tree with its
        // animated transform (builder exempts nulls from the opacity-0 skip).
        let at_zero = player
            .get_layer_transform("REF:dock")
            .expect("null layer must be queryable");

        // Its animated position must sample per-frame.
        assert!(player.set_frame(45.0).is_ok());
        assert!(player.render().is_ok());
        let at_45 = player.get_layer_transform("REF:dock").unwrap();
        assert_ne!(at_zero, at_45);

        // Regular layers are queryable too; unknown names are None.
        assert!(player.get_layer_transform("bell").is_some());
        assert!(player.get_layer_opacity("bell").is_some());
        assert!(player.get_layer_transform("no-such-layer").is_none());

        // Query returns pristine values even while user overrides are active.
        let pristine = player.get_layer_transform("bell").unwrap();
        let shift = vec![1.0, 0.0, 300.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        assert!(player.set_layer_transform("bell", shift).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(player.get_layer_transform("bell").unwrap(), pristine);
    }

    #[test]
    fn scene_clip_and_mask_change_pixels() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();
        let baseline_nonzero = baseline.iter().filter(|&&px| px != 0).count();

        // Clip to the center quarter (canvas px): fewer pixels survive.
        assert!(player.set_clip_rect(25.0, 25.0, 50.0, 50.0, 0.0, 0.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);
        let clipped_nonzero = buffer.iter().filter(|&&px| px != 0).count();
        assert!(clipped_nonzero < baseline_nonzero, "clip should discard pixels");

        // Clip survives a frame change (the clipper lives on our wrapping
        // scene, which ThorVG never rebuilds).
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        assert!(
            buffer.iter().filter(|&&px| px != 0).count() < baseline_nonzero,
            "clip must persist across frames"
        );
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());

        assert!(player.set_clip_circle(50.0, 50.0, 30.0, 30.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.clear_clip().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // Feathered spotlight vs inverse cutout: both differ from the
        // baseline and from each other.
        assert!(player.set_spot_mask(50.0, 50.0, 35.0, 0.5, false).is_ok());
        assert!(player.render().is_ok());
        let spotlight = buffer.clone();
        assert_ne!(spotlight, baseline);

        assert!(player.set_spot_mask(50.0, 50.0, 35.0, 0.5, true).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);
        assert_ne!(buffer, spotlight);

        assert!(player.clear_mask().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
    }

    #[test]
    fn layer_clip_survives_frames_and_clears() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();

        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        let clean_frame22 = buffer.clone();
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());

        // Layer clip coordinates are composition units (test.json is a
        // 1500-unit comp): keep only the left half of layer "R".
        assert!(player.set_layer_clip_rect("R", 0.0, 0.0, 750.0, 1500.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        // Survives the layer-scene rebuild on frame change.
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, clean_frame22);
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());

        // w <= 0 removes the clip — including on a paused (non-rebuilt) scene.
        assert!(player.set_layer_clip_rect("R", 0.0, 0.0, 0.0, 0.0).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // clear_layer_props also drops an active clip.
        assert!(player.set_layer_clip_rect("R", 0.0, 0.0, 750.0, 1500.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);
        assert!(player.clear_layer_props("R").is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
    }

    #[test]
    fn path_clip_scene_and_layer() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();
        let baseline_nonzero = baseline.iter().filter(|&&px| px != 0).count();

        // Upper-left triangle in canvas px: M 0,0 L 100,0 L 0,100 Z.
        let tri_cmds = vec![1u8, 2, 2, 0];
        let tri_pts = vec![0.0f32, 0.0, 100.0, 0.0, 0.0, 100.0];
        assert!(player.set_clip_path(tri_cmds.clone(), tri_pts).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);
        assert!(
            buffer.iter().filter(|&&px| px != 0).count() < baseline_nonzero,
            "path clip should discard pixels"
        );
        let triangle = buffer.clone();

        // A path clip is not a rect clip of the same bounding box.
        assert!(player.set_clip_rect(0.0, 0.0, 100.0, 100.0, 0.0, 0.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, triangle);

        // Survives a frame change and clears back to baseline.
        let tri_pts = vec![0.0f32, 0.0, 100.0, 0.0, 0.0, 100.0];
        assert!(player.set_clip_path(tri_cmds, tri_pts).is_ok());
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        assert!(
            buffer.iter().filter(|&&px| px != 0).count() < baseline_nonzero,
            "path clip must persist across frames"
        );
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        assert!(player.clear_clip().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // Invalid paths error without touching state: no leading MoveTo,
        // wrong point count, unknown command, non-finite coordinate.
        assert!(player.set_clip_path(vec![2], vec![0.0, 0.0]).is_err());
        assert!(player.set_clip_path(vec![1, 2], vec![0.0, 0.0]).is_err());
        assert!(player.set_clip_path(vec![1, 9], vec![0.0, 0.0, 1.0, 1.0]).is_err());
        assert!(player.set_clip_path(vec![1], vec![f32::NAN, 0.0]).is_err());
        // Nothing changed, so render may legitimately report nothing-to-do.
        let _ = player.render();
        assert_eq!(buffer, baseline);

        // Layer path clip in composition units (1500-unit comp): keep the
        // upper-left triangle of layer "R".
        let layer_cmds = vec![1u8, 2, 2, 0];
        let layer_pts = vec![0.0f32, 0.0, 1500.0, 0.0, 0.0, 1500.0];
        assert!(player
            .set_layer_clip_path("R", layer_cmds, layer_pts)
            .is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        // Empty cmds removes the layer clip.
        assert!(player.set_layer_clip_path("R", vec![], vec![]).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
    }

    #[test]
    fn overlays_render_and_survive_reload() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();

        // Filled square over the canvas center, above the animation.
        let square_cmds = vec![1u8, 2, 2, 2, 0];
        let square_pts = vec![30.0f32, 30.0, 70.0, 30.0, 70.0, 70.0, 30.0, 70.0];
        let above = player.add_overlay(false).unwrap();
        assert!(player
            .set_overlay_path(above, square_cmds.clone(), square_pts.clone())
            .is_ok());
        assert!(player.set_overlay_fill(above, 220, 40, 40, 255).is_ok());
        assert!(player.render().is_ok());
        let above_buf = buffer.clone();
        assert_ne!(above_buf, baseline);

        assert!(player.remove_overlay(above).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline, "remove must restore the baseline");

        // Frame 21 is opaque edge-to-edge, which would hide anything below
        // the picture — shrink the art so empty canvas surrounds it, then
        // z-order a full-canvas square on both sides of it. Overlays live in
        // the wrapping scene, so the picture transform doesn't move them.
        let shrink = vec![0.6, 0.0, 20.0, 0.0, 0.6, 20.0, 0.0, 0.0, 1.0];
        assert!(player.set_transform(shrink).is_ok());
        assert!(player.render().is_ok());
        let shrunk = buffer.clone();
        assert_ne!(shrunk, baseline);

        let full_cmds = vec![1u8, 2, 2, 2, 0];
        let full_pts = vec![0.0f32, 0.0, 100.0, 0.0, 100.0, 100.0, 0.0, 100.0];
        let below = player.add_overlay(true).unwrap();
        assert!(player
            .set_overlay_path(below, full_cmds.clone(), full_pts.clone())
            .is_ok());
        assert!(player.set_overlay_fill(below, 220, 40, 40, 255).is_ok());
        assert!(player.render().is_ok());
        let below_buf = buffer.clone();
        assert_ne!(below_buf, shrunk, "below fills the empty surround");

        let above2 = player.add_overlay(false).unwrap();
        assert!(player.set_overlay_path(above2, full_cmds, full_pts).is_ok());
        assert!(player.set_overlay_fill(above2, 220, 40, 40, 255).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, below_buf, "above covers the art, below does not");

        assert!(player.remove_overlay(below).is_ok());
        assert!(player.remove_overlay(above2).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, shrunk);
        let identity = vec![1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        assert!(player.set_transform(identity).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // Mutating geometry / transform re-renders without a frame change.
        let id = player.add_overlay(false).unwrap();
        assert!(player
            .set_overlay_path(id, square_cmds.clone(), square_pts.clone())
            .is_ok());
        assert!(player.set_overlay_fill(id, 40, 220, 40, 255).is_ok());
        assert!(player.render().is_ok());
        let green_square = buffer.clone();
        assert_ne!(green_square, baseline);

        let tri_pts = vec![30.0f32, 30.0, 70.0, 30.0, 30.0, 70.0];
        assert!(player.set_overlay_path(id, vec![1, 2, 2, 0], tri_pts).is_ok());
        assert!(player.render().is_ok());
        let green_triangle = buffer.clone();
        assert_ne!(green_triangle, green_square);

        let shift = vec![1.0f32, 0.0, 15.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        assert!(player.set_overlay_transform(id, shift).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, green_triangle);

        // Stroke-only shapes render too; width 0 removes the stroke.
        assert!(player.set_overlay_fill(id, 0, 0, 0, 0).is_ok());
        assert!(player.set_overlay_stroke(id, 6.0, 40, 40, 220, 255).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        // Overlays replay across an animation reload.
        let before_reload = buffer.clone();
        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, before_reload, "overlay must survive reload");

        // Unknown ids error; invalid path errors without killing the overlay.
        assert!(player.set_overlay_fill(999, 1, 2, 3, 4).is_err());
        assert!(player.remove_overlay(999).is_err());
        assert!(player.set_overlay_path(id, vec![2], vec![0.0, 0.0]).is_err());

        assert!(player.clear_overlays().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
    }

    #[test]
    fn effects_persist_across_reload() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let clean = buffer.clone();

        assert!(player.add_gaussian_blur(8.0, 0, 0, 100).is_ok());
        assert!(player.render().is_ok());
        let blurred = buffer.clone();
        assert_ne!(blurred, clean);

        // Reload: stored effects replay onto the new animation automatically.
        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, clean);

        // clear_effects also persists (empty stack) across reloads.
        assert!(player.clear_effects().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, clean);
    }

    #[test]
    fn reload_cycle_survives_scene_wrap() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);

        assert!(player.set_opacity(128).is_ok());
        assert!(player.add_gaussian_blur(4.0, 0, 0, 50).is_ok());
        assert!(player.render().is_ok());

        // Reload drops the wrapped TvgAnimation while the canvas holds the scene.
        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        let path2 = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path2).is_ok());
        assert!(buffer.iter().any(|&px| px != 0));
    }

    #[test]
    fn precise_hit_test_and_aabb() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();

        // Derive layer "E"'s actual pixel coverage by hiding it.
        assert!(player.set_layer_visible("E", false).is_ok());
        assert!(player.render().is_ok());
        let hidden = buffer.clone();
        let coverage: Vec<usize> = (0..buffer.len())
            .filter(|&i| hidden[i] != baseline[i])
            .collect();
        assert!(!coverage.is_empty(), "hiding E should change pixels");

        // Hidden layers stay hit-testable (render data is prepared; hidden is
        // draw-phase only) — the invisible-drop-zone lever.
        let covered = coverage[coverage.len() / 2];
        let (cx, cy) = ((covered % WIDTH as usize) as f32, (covered / WIDTH as usize) as f32);
        assert!(player.hit_test_precise("E", cx, cy, false));
        assert!(!player.hit_test_precise("E", cx, cy, true));

        assert!(player.set_layer_visible("E", true).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
        assert!(player.hit_test_precise("E", cx, cy, true));

        // AABB contains every covered pixel, in canvas space.
        let aabb = player.get_layer_aabb("E").expect("E has an AABB");
        let (ax, ay, aw, ah) = (aabb[0], aabb[1], aabb[2], aabb[3]);
        assert!(aw > 0.0 && ah > 0.0);
        for &i in &coverage {
            let (px, py) = ((i % WIDTH as usize) as f32, (i / WIDTH as usize) as f32);
            assert!(
                px >= ax - 1.0 && px <= ax + aw + 1.0 && py >= ay - 1.0 && py <= ay + ah + 1.0,
                "covered pixel ({px}, {py}) outside AABB {aabb:?}"
            );
        }

        // Precise beats the box: some point inside the AABB isn't covered.
        let mut uncovered = None;
        'scan: for y in (ay as usize)..((ay + ah) as usize).min(HEIGHT as usize) {
            for x in (ax as usize)..((ax + aw) as usize).min(WIDTH as usize) {
                if !coverage.contains(&(y * WIDTH as usize + x)) {
                    uncovered = Some((x as f32, y as f32));
                    break 'scan;
                }
            }
        }
        let (ux, uy) = uncovered.expect("free path should not fill its AABB");
        assert!(!player.hit_test_precise("E", ux, uy, false));

        // Region test: the whole AABB intersects; a far-away region doesn't.
        assert!(player.intersects_layer("E", ax, ay, aw, ah, false));
        assert!(!player.intersects_layer("E", ax + aw + 5.0, ay + ah + 5.0, 3.0, 3.0, false));
        assert!(!player.intersects_layer("E", 0.0, 0.0, 0.0, 5.0, false));

        // Unknown layers: no AABB, no hit.
        assert!(player.get_layer_aabb("nope").is_none());
        assert!(!player.hit_test_precise("nope", cx, cy, false));
    }

    #[test]
    fn layer_clones_render_and_replay() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();

        // A fresh clone overlaps its source exactly; moving it reveals it.
        let id = player.add_layer_clone("E", false).unwrap();
        assert!(player
            .set_clone_transform(id, vec![1.0, 0.0, 25.0, 0.0, 1.0, 10.0, 0.0, 0.0, 1.0])
            .is_ok());
        assert!(player.render().is_ok());
        let moved = buffer.clone();
        assert_ne!(moved, baseline);

        assert!(player.set_clone_opacity(id, 110).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, moved);

        // The clone is a frozen snapshot: the source animates, the copy stays.
        assert!(player.set_frame(30.0).is_ok());
        assert!(player.render().is_ok());
        assert!(buffer.iter().any(|&px| px != 0));

        assert!(player.set_frame(21.0).is_ok());
        assert!(player.remove_clone(id).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        assert!(player.remove_clone(id).is_err());
        assert!(player.add_layer_clone("nope", false).is_err());

        // Clones replay across reloads (re-duplicated on the fresh tree).
        // Above the art — frame 21 is opaque edge-to-edge, so a below-clone
        // would be fully occluded.
        let id2 = player.add_layer_clone("E", false).unwrap();
        assert!(player
            .set_clone_transform(id2, vec![1.0, 0.0, -20.0, 0.0, 1.0, -15.0, 0.0, 0.0, 1.0])
            .is_ok());
        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline, "replayed clone should still render");

        assert!(player.clear_clones().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
    }

    #[test]
    fn overlay_gradients_and_dash() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();

        let rect_cmds = vec![1u8, 2, 2, 2, 0];
        let rect_pts = vec![10.0f32, 10.0, 90.0, 10.0, 90.0, 60.0, 10.0, 60.0];
        let id = player.add_overlay(false).unwrap();
        assert!(player.set_overlay_path(id, rect_cmds, rect_pts).is_ok());
        assert!(player.set_overlay_fill(id, 200, 30, 30, 255).is_ok());
        assert!(player.render().is_ok());
        let solid = buffer.clone();
        assert_ne!(solid, baseline);
        // A solid fill is horizontally uniform inside the rect...
        let row = 35 * WIDTH as usize;
        assert_eq!(solid[row + 20], solid[row + 80]);

        let stops = vec![0.0, 255.0, 0.0, 0.0, 255.0, 1.0, 0.0, 0.0, 255.0, 255.0];
        assert!(player
            .set_overlay_fill_linear(id, 10.0, 0.0, 90.0, 0.0, stops.clone())
            .is_ok());
        assert!(player.render().is_ok());
        let linear = buffer.clone();
        assert_ne!(linear, solid);
        // ...a horizontal gradient is not.
        assert_ne!(linear[row + 20], linear[row + 80]);

        assert!(player
            .set_overlay_fill_radial(id, 50.0, 35.0, 40.0, stops)
            .is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, linear);

        // Switching back to a solid fill fully clears the gradient.
        assert!(player.set_overlay_fill(id, 200, 30, 30, 255).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, solid);

        // Dash: a stroked line loses pixels when dashed, and an empty pattern
        // restores the solid stroke byte-exactly.
        let line = player.add_overlay(false).unwrap();
        assert!(player
            .set_overlay_path(line, vec![1, 2], vec![10.0, 80.0, 90.0, 80.0])
            .is_ok());
        assert!(player.set_overlay_stroke(line, 5.0, 30, 200, 30, 255).is_ok());
        assert!(player.render().is_ok());
        let solid_stroke = buffer.clone();
        let stroke_px = (0..buffer.len()).filter(|&i| solid_stroke[i] != solid[i]).count();
        assert!(stroke_px > 0);

        assert!(player.set_overlay_stroke_dash(line, vec![6.0, 5.0]).is_ok());
        assert!(player.render().is_ok());
        let dashed_px = (0..buffer.len()).filter(|&i| buffer[i] != solid[i]).count();
        assert!(dashed_px > 0 && dashed_px < stroke_px, "dashing should drop pixels");

        assert!(player.set_overlay_stroke_dash(line, vec![]).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, solid_stroke);

        // Validation: bad stops/patterns error and leave pixels untouched.
        assert!(player
            .set_overlay_fill_linear(id, 0.0, 0.0, 1.0, 1.0, vec![])
            .is_err());
        assert!(player
            .set_overlay_fill_linear(id, 0.0, 0.0, 1.0, 1.0, vec![0.0, 1.0])
            .is_err());
        assert!(player
            .set_overlay_fill_radial(id, 50.0, 50.0, 0.0, vec![0.0, 1.0, 1.0, 1.0, 255.0])
            .is_err());
        assert!(player
            .set_overlay_fill_linear(id, f32::NAN, 0.0, 1.0, 1.0, vec![0.0, 1.0, 1.0, 1.0, 255.0])
            .is_err());
        assert!(player.set_overlay_stroke_dash(line, vec![-1.0, 4.0]).is_err());
        assert!(player.set_overlay_stroke_dash(line, vec![0.0, 0.0]).is_err());
        let _ = player.render();
        assert_eq!(buffer, solid_stroke);

        assert!(player.clear_overlays().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);
    }
}
