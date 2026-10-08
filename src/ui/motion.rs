//! Interface motion, and the Reduce motion setting that stills it.
//!
//! Every animation goes through here: with Reduce motion on, each one is
//! at its end the moment it starts.

use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Context, Id};

/// Screenshot runs hold every animation at its end, so a capture never
/// catches one halfway.
static STILL: AtomicBool = AtomicBool::new(false);

const REDUCED_ID: &str = "motion-reduced";

/// Still every animation for the rest of the process.
pub fn hold_still() {
    STILL.store(true, Ordering::Relaxed);
}

/// Record this frame's Reduce motion setting.
pub fn set_reduced(ctx: &Context, reduced: bool) {
    ctx.data_mut(|data| data.insert_temp(Id::new(REDUCED_ID), reduced));
}

/// Whether animations snap to their end.
pub fn reduced(ctx: &Context) -> bool {
    STILL.load(Ordering::Relaxed)
        || ctx
            .data(|data| data.get_temp::<bool>(Id::new(REDUCED_ID)))
            .unwrap_or(false)
}

/// [`Context::animate_bool_with_time`], eased out, or `value` at once.
pub fn animate_bool(ctx: &Context, id: Id, value: bool, seconds: f32) -> f32 {
    if reduced(ctx) {
        // Kept in step, so turning the setting off later starts from here.
        ctx.animate_bool_with_time(id, value, 0.0);
        return if value { 1.0 } else { 0.0 };
    }
    egui::emath::easing::cubic_out(ctx.animate_bool_with_time(id, value, seconds))
}

/// [`Context::animate_value_with_time`], or `value` at once.
pub fn animate_value(ctx: &Context, id: Id, value: f32, seconds: f32) -> f32 {
    if reduced(ctx) {
        ctx.animate_value_with_time(id, value, 0.0);
        return value;
    }
    ctx.animate_value_with_time(id, value, seconds)
}

/// Begin a one-off animation under `id`, such as a pop, from now.
pub fn start(ctx: &Context, id: Id) {
    if reduced(ctx) {
        return;
    }
    let now = ctx.input(|input| input.time);
    ctx.data_mut(|data| data.insert_temp(id.with(START), now));
}

const START: &str = "motion-start";

/// How far through its `seconds` the one-off animation under `id` is, from
/// 0 to below 1, or `None` once it has finished or when it never started.
/// Asks for the next frame while it runs.
pub fn progress(ctx: &Context, id: Id, seconds: f32) -> Option<f32> {
    let key = id.with(START);
    let started = ctx.data(|data| data.get_temp::<f64>(key))?;
    let now = ctx.input(|input| input.time);
    let t = ((now - started) as f32 / seconds.max(f32::EPSILON)).max(0.0);
    if t >= 1.0 || reduced(ctx) {
        ctx.data_mut(|data| data.remove::<f64>(key));
        return None;
    }
    ctx.request_repaint();
    Some(t)
}

/// How long a heart takes to pop when a song is liked.
const HEART_POP_SECONDS: f32 = 0.2;

fn heart_pop_id(uri: &str) -> Id {
    Id::new(("heart-pop", uri))
}

/// Pop every heart of `uri`: the song was just liked.
pub fn pop_heart(ctx: &Context, uri: &str) {
    start(ctx, heart_pop_id(uri));
}

/// How large `uri`'s heart is drawn: past its size and back while it pops.
pub fn heart_scale(ctx: &Context, uri: &str) -> f32 {
    progress(ctx, heart_pop_id(uri), HEART_POP_SECONDS)
        .map_or(1.0, |t| 1.0 + 0.35 * (std::f32::consts::PI * t).sin())
}

const BARS: usize = 4;
const BARS_CLOCK: &str = "playing-bars-clock";
/// How often moving bars are redrawn: thirty times a second is smooth at
/// their size.
const BARS_FRAME: std::time::Duration = std::time::Duration::from_millis(33);

/// Each bar's height, from 0 to 1, `t` seconds into the song's dance.
/// Every bar mixes two slow waves of its own, so they never move in step.
pub fn bar_heights(t: f64) -> [f32; BARS] {
    const WAVES: [(f64, f64, f64, f64); BARS] = [
        (7.1, 0.0, 3.3, 1.9),
        (5.3, 2.1, 8.9, 0.4),
        (8.7, 4.0, 4.1, 2.8),
        (6.2, 1.2, 7.4, 5.1),
    ];
    WAVES.map(|(a, pa, b, pb)| {
        let wave = ((t * a + pa).sin() + (t * b + pb).sin()) * 0.25 + 0.5;
        (0.25 + 0.75 * wave) as f32
    })
}

/// The heights still bars show under Reduce motion.
const STILL_BARS: [f32; BARS] = [0.55, 1.0, 0.4, 0.75];

/// Small bars in `rect` that dance while `moving`, as the playing song's
/// mark in a track list. They hold where they were while the song is
/// paused, and stand still under Reduce motion. Moving bars ask for the
/// next frame only while they are on screen.
pub fn playing_bars(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32, moving: bool) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let ctx = ui.ctx();
    let heights = if reduced(ctx) {
        STILL_BARS
    } else {
        let clock = Id::new(BARS_CLOCK);
        let t = if moving {
            let now = ctx.input(|input| input.time);
            ctx.data_mut(|data| data.insert_temp(clock, now));
            ctx.request_repaint_after(BARS_FRAME);
            now
        } else {
            ctx.data(|data| data.get_temp::<f64>(clock)).unwrap_or(0.0)
        };
        bar_heights(t)
    };
    let (width, gap, tallest) = (2.5, 2.0, 13.0);
    let span = BARS as f32 * width + (BARS - 1) as f32 * gap;
    let foot = rect.center().y + tallest / 2.0;
    let mut left = rect.center().x - span / 2.0;
    for height in heights {
        let bar = egui::Rect::from_min_max(
            egui::pos2(left, foot - tallest * height),
            egui::pos2(left + width, foot),
        );
        ui.painter().rect_filled(bar, 1.0, color);
        left += width + gap;
    }
}

/// Hover states a demo capture asks for, since a screenshot has no pointer.
#[cfg(feature = "demo")]
static FORCED_HOVER: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

/// Show the first `what` drawn each frame as hovered, for demo captures.
#[cfg(feature = "demo")]
pub fn force_hover(what: &'static str) {
    FORCED_HOVER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push(what);
}

/// Whether a demo capture shows this, the first `what` of the frame, as
/// hovered. Always false outside demo builds.
pub fn forced_hover(ctx: &Context, what: &'static str) -> bool {
    #[cfg(feature = "demo")]
    {
        let wanted = FORCED_HOVER
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains(&what);
        if !wanted {
            return false;
        }
        let pass = ctx.cumulative_pass_nr();
        let id = Id::new(("motion-forced-hover", what));
        let first = ctx.data(|data| data.get_temp::<u64>(id)) != Some(pass);
        ctx.data_mut(|data| data.insert_temp(id, pass));
        first
    }
    #[cfg(not(feature = "demo"))]
    {
        let _ = (ctx, what);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &Context, time: f64, reduced: bool, mut run: impl FnMut(&Context)) {
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                set_reduced(ui.ctx(), reduced);
                run(ui.ctx());
            },
        );
        output.textures_delta.clear();
    }

    #[test]
    fn reduce_motion_snaps_every_animation_to_its_end() {
        let ctx = Context::default();
        let id = Id::new("fade");
        let mut seen = Vec::new();
        frame(&ctx, 0.0, true, |ctx| {
            seen.push(animate_bool(ctx, id, false, 0.3));
        });
        frame(&ctx, 0.01, true, |ctx| {
            seen.push(animate_bool(ctx, id, true, 0.3));
            seen.push(animate_value(ctx, id.with(1), 4.0, 0.3));
        });
        frame(&ctx, 0.02, true, |ctx| {
            seen.push(animate_value(ctx, id.with(1), 9.0, 0.3));
            start(ctx, id.with(2));
            assert_eq!(progress(ctx, id.with(2), 0.2), None);
        });
        assert_eq!(seen, [0.0, 1.0, 4.0, 9.0]);
    }

    /// A liked song's heart swells and settles back within its pop, and
    /// only that song's; Reduce motion leaves it still.
    #[test]
    fn a_liked_hearts_pop_swells_and_settles() {
        for reduced in [false, true] {
            let ctx = Context::default();
            frame(&ctx, 0.0, reduced, |ctx| pop_heart(ctx, "spotify:track:a"));
            let mut scales = Vec::new();
            for time in [0.1, 0.15, 0.5] {
                frame(&ctx, time, reduced, |ctx| {
                    scales.push(heart_scale(ctx, "spotify:track:a"));
                    assert_eq!(heart_scale(ctx, "spotify:track:b"), 1.0);
                });
            }
            if reduced {
                assert_eq!(scales, [1.0, 1.0, 1.0]);
            } else {
                assert!(scales[0] > 1.3, "{scales:?}");
                assert!(scales[1] > 1.0 && scales[1] < scales[0], "{scales:?}");
                assert_eq!(scales[2], 1.0);
            }
        }
    }

    /// Bars stay within their cell and keep moving apart from each other.
    #[test]
    fn playing_bars_dance_within_their_cell() {
        let mut changed = false;
        let first = bar_heights(0.0);
        for step in 0..600 {
            let heights = bar_heights(f64::from(step) * 0.033);
            assert!(
                heights.iter().all(|h| (0.25..=1.0).contains(h)),
                "{heights:?}"
            );
            changed |= heights != first;
        }
        assert!(changed);
    }

    /// Moving bars ask for the next frame; paused or still bars, and bars
    /// out of sight, leave the app idle.
    #[test]
    fn only_moving_bars_on_screen_keep_the_app_drawing() {
        let delay = |moving: bool, reduced: bool, visible: bool| {
            let ctx = Context::default();
            let mut delay = std::time::Duration::ZERO;
            // egui draws its first frames again regardless; measure after.
            for _ in 0..3 {
                let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    set_reduced(ui.ctx(), reduced);
                    let rect = if visible {
                        egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(20.0, 20.0))
                    } else {
                        egui::Rect::from_min_size(
                            egui::pos2(-500.0, -500.0),
                            egui::vec2(20.0, 20.0),
                        )
                    };
                    playing_bars(ui, rect, egui::Color32::WHITE, moving);
                });
                output.textures_delta.clear();
                delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
            }
            delay
        };
        assert!(delay(true, false, true) <= BARS_FRAME);
        assert_eq!(delay(false, false, true), std::time::Duration::MAX);
        assert_eq!(delay(true, true, true), std::time::Duration::MAX);
        assert_eq!(delay(true, false, false), std::time::Duration::MAX);
    }

    /// The seek bar eases thicker under the pointer and back once it
    /// leaves; Reduce motion makes the change at once.
    #[test]
    fn a_hovered_slider_eases_thicker_and_back() {
        for reduced in [false, true] {
            let ctx = Context::default();
            let palette = crate::theme::Palette::dark();
            let draw = |time: f64, pointer: egui::Pos2| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        time: Some(time),
                        events: vec![egui::Event::PointerMoved(pointer)],
                        ..Default::default()
                    },
                    |ui| {
                        set_reduced(ui.ctx(), reduced);
                        crate::ui::widgets::thin_slider(
                            ui,
                            &palette,
                            Id::new("seek-slider-test"),
                            "Position",
                            0.5,
                            300.0,
                            None,
                        );
                    },
                );
                output.textures_delta.clear();
                output
                    .shapes
                    .iter()
                    .find_map(|clipped| match &clipped.shape {
                        egui::epaint::Shape::Rect(rect) if rect.rect.width() > 299.0 => {
                            Some(rect.rect.height())
                        }
                        _ => None,
                    })
                    .expect("the slider's track")
            };
            let away = egui::pos2(900.0, 900.0);
            assert_eq!(draw(0.0, away), 4.0);
            let over = egui::pos2(150.0, 16.0);
            let early = draw(0.02, over);
            if reduced {
                assert_eq!(early, 6.0);
            } else {
                assert!(early > 4.0 && early < 6.0, "{early}");
            }
            assert_eq!(draw(1.0, over), 6.0);
            draw(1.02, away);
            assert_eq!(draw(2.0, away), 4.0);
        }
    }

    #[test]
    fn without_reduce_motion_animations_take_their_time() {
        let ctx = Context::default();
        let id = Id::new("fade");
        let mut middle = 0.0;
        frame(&ctx, 0.0, false, |ctx| {
            animate_bool(ctx, id, false, 0.3);
        });
        frame(&ctx, 0.01, false, |ctx| {
            middle = animate_bool(ctx, id, true, 0.3);
            start(ctx, id.with(2));
        });
        assert!(middle > 0.0 && middle < 1.0, "{middle}");
        let mut pop = None;
        frame(&ctx, 0.1, false, |ctx| pop = progress(ctx, id.with(2), 0.2));
        let pop = pop.expect("halfway through");
        assert!((pop - 0.45).abs() < 0.01, "{pop}");
        frame(&ctx, 0.5, false, |ctx| {
            assert_eq!(progress(ctx, id.with(2), 0.2), None);
        });
    }
}
