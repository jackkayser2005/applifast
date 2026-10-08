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

/// `animation`, or an instant scroll under Reduce motion.
pub fn scroll(
    ctx: &Context,
    animation: egui::style::ScrollAnimation,
) -> egui::style::ScrollAnimation {
    if reduced(ctx) {
        egui::style::ScrollAnimation::none()
    } else {
        animation
    }
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

/// How long a newly opened page takes to fade fully in.
const PAGE_FADE_SECONDS: f32 = 0.14;
/// How visible a page is the moment it opens: enough to show at once what
/// was clicked, while the fade still marks the change.
const PAGE_FADE_FROM: f32 = 0.35;

/// How opaque to draw the page `key` this frame: it fades in briefly when
/// it replaces another page, and is fully drawn otherwise.
pub fn page_opacity(ctx: &Context, key: &str) -> f32 {
    let shown = Id::new("motion-page-shown");
    let fade = Id::new("motion-page-fade");
    let previous = ctx.data(|data| data.get_temp::<String>(shown));
    if previous.as_deref() != Some(key) {
        ctx.data_mut(|data| data.insert_temp(shown, key.to_string()));
        if previous.is_some() {
            start(ctx, fade);
        }
    }
    progress(ctx, fade, PAGE_FADE_SECONDS).map_or(1.0, |t| {
        PAGE_FADE_FROM + (1.0 - PAGE_FADE_FROM) * egui::emath::easing::cubic_out(t)
    })
}

/// How long a newly arrived row takes to ease into place.
const ARRIVAL_SECONDS: f32 = 0.22;

/// Which rows of a list are new, and since when.
#[derive(Clone, Default)]
struct Arrivals {
    known: std::collections::HashSet<String>,
    since: std::collections::HashMap<String, f64>,
    /// The frame that last drew the list. A list out of sight for a while
    /// starts afresh rather than easing in everything added meanwhile.
    pass: Option<u64>,
}

impl Arrivals {
    fn update<'a>(&mut self, keys: impl Iterator<Item = &'a str>, pass: u64, now: f64) {
        let keys: std::collections::HashSet<String> = keys.map(str::to_string).collect();
        if self.pass.is_some_and(|last| last + 1 >= pass) {
            for key in keys.difference(&self.known) {
                self.since.insert(key.clone(), now);
            }
        } else {
            self.since.clear();
        }
        self.since
            .retain(|key, since| keys.contains(key) && now - *since < f64::from(ARRIVAL_SECONDS));
        self.known = keys;
        self.pass = Some(pass);
    }
}

/// Note the rows the list under `id` holds this frame, by key. Rows that
/// were not there the frame before ease in; see [`arrival`].
pub fn note_rows<'a>(ctx: &Context, id: Id, keys: impl Iterator<Item = &'a str>) {
    let mut arrivals = ctx
        .data(|data| data.get_temp::<std::sync::Arc<Arrivals>>(id))
        .unwrap_or_default();
    let (pass, now) = (ctx.cumulative_pass_nr(), ctx.input(|input| input.time));
    let changed = std::sync::Arc::make_mut(&mut arrivals);
    changed.update(keys, pass, now);
    if reduced(ctx) {
        changed.since.clear();
    }
    ctx.data_mut(|data| data.insert_temp(id, arrivals));
}

/// How far the row `key` of the list under `id` has eased in, from 0 just
/// added to 1 settled.
pub fn arrival(ctx: &Context, id: Id, key: &str) -> f32 {
    let since = ctx.data(|data| {
        data.get_temp::<std::sync::Arc<Arrivals>>(id)
            .and_then(|arrivals| arrivals.since.get(key).copied())
    });
    let Some(since) = since else {
        return 1.0;
    };
    let t = ((ctx.input(|input| input.time) - since) as f32 / ARRIVAL_SECONDS).clamp(0.0, 1.0);
    if t < 1.0 {
        ctx.request_repaint();
    }
    egui::emath::easing::cubic_out(t)
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

/// Decorative pulses from the playhead, not an audio spectrum. Both players
/// share the phase, including after a seek. Pausing freezes it; volume has no
/// effect. Only visible playback asks for another frame.
pub fn ambient_pulse(
    ui: &egui::Ui,
    rect: egui::Rect,
    color: egui::Color32,
    position_ms: u32,
    playing: bool,
) {
    if !ui.is_rect_visible(rect)
        || ui
            .ctx()
            .input(|input| input.viewport().minimized == Some(true))
    {
        return;
    }
    let still = reduced(ui.ctx());
    let phase = if still {
        0.0
    } else {
        f64::from(position_ms) / 1000.0
    };
    let painter = ui.painter().with_clip_rect(rect);
    const BANDS: usize = 24;
    let width = rect.width() / BANDS as f32;
    for band in 0..BANDS {
        let heights = bar_heights(phase + band as f64 * 0.16);
        let height = rect.height() * (0.15 + 0.7 * heights[band % BARS]);
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left() + width * band as f32, rect.bottom() - height),
            egui::vec2(width * 0.7, height),
        );
        painter.rect_filled(bar, width.min(4.0), color.gamma_multiply(0.14));
    }
    if playing && !still {
        ui.ctx().request_repaint_after(BARS_FRAME);
    }
}

/// Small bars in `rect` that dance while `moving`, as the playing song's
/// mark in a track list. They hold where they were while the song is
/// paused, and stand still under Reduce motion. Moving bars ask for the
/// next frame only while they are on screen.
pub fn playing_bars(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32, moving: bool) {
    if !ui.is_rect_visible(rect)
        || ui
            .ctx()
            .input(|input| input.viewport().minimized == Some(true))
    {
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

    #[test]
    fn ambient_pulse_follows_the_playhead_and_only_repaints_visible_playback() {
        let draw = |position_ms, playing, reduced, visible| {
            let ctx = Context::default();
            let mut bars = Vec::new();
            let mut delay = std::time::Duration::ZERO;
            for time in 0..3 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        time: Some(f64::from(time)),
                        ..Default::default()
                    },
                    |ui| {
                        set_reduced(ui.ctx(), reduced);
                        let rect = egui::Rect::from_min_size(
                            if visible {
                                egui::pos2(10.0, 10.0)
                            } else {
                                egui::pos2(-500.0, -500.0)
                            },
                            egui::vec2(240.0, 80.0),
                        );
                        ambient_pulse(ui, rect, egui::Color32::WHITE, position_ms, playing);
                    },
                );
                output.textures_delta.clear();
                bars = output
                    .shapes
                    .iter()
                    .filter_map(|shape| match &shape.shape {
                        egui::epaint::Shape::Rect(rect) => Some(rect.rect),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
            }
            (bars, delay)
        };
        let (playing, delay) = draw(2000, true, false, true);
        assert_eq!(playing.len(), 24);
        assert!(delay <= BARS_FRAME);
        let (paused, delay) = draw(2000, false, false, true);
        assert_eq!(playing, paused);
        assert_eq!(delay, std::time::Duration::MAX);
        assert_ne!(
            playing,
            draw(4000, true, false, true).0,
            "seeking changes the phase"
        );
        let (still, delay) = draw(2000, true, true, true);
        assert_eq!(still, draw(4000, true, true, true).0);
        assert_eq!(delay, std::time::Duration::MAX);
        let (hidden, delay) = draw(2000, true, false, false);
        assert!(hidden.is_empty());
        assert_eq!(delay, std::time::Duration::MAX);
    }

    #[test]
    fn minimized_views_do_not_animate_pulses_or_playing_indicators() {
        let ctx = Context::default();
        for frame in 0..3 {
            let mut input = egui::RawInput::default();
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .minimized = Some(true);
            let mut output = ctx.run_ui(input, |ui| {
                let rect =
                    egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(240.0, 80.0));
                ambient_pulse(ui, rect, egui::Color32::WHITE, 2000, true);
                playing_bars(ui, rect, egui::Color32::WHITE, true);
            });
            output.textures_delta.clear();
            assert!(
                !output
                    .shapes
                    .iter()
                    .any(|shape| matches!(shape.shape, egui::epaint::Shape::Rect(_)))
            );
            if frame == 2 {
                assert_eq!(
                    output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
                    std::time::Duration::MAX
                );
            }
        }
    }

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

    /// A page fades in when it replaces another, never when the app opens
    /// on it or while it stays; Reduce motion shows it at once.
    #[test]
    fn a_new_page_fades_in_unless_motion_is_reduced() {
        for reduced in [false, true] {
            let ctx = Context::default();
            let mut seen = Vec::new();
            for (time, page) in [
                (0.0, "home"),
                (0.5, "home"),
                (1.0, "album"),
                (1.07, "album"),
            ] {
                frame(&ctx, time, reduced, |ctx| {
                    seen.push(page_opacity(ctx, page))
                });
            }
            frame(&ctx, 2.0, reduced, |ctx| {
                seen.push(page_opacity(ctx, "album"))
            });
            if reduced {
                assert_eq!(seen, [1.0; 5]);
            } else {
                assert_eq!(seen[..2], [1.0, 1.0]);
                assert_eq!(seen[2], PAGE_FADE_FROM);
                assert!(seen[3] > PAGE_FADE_FROM && seen[3] < 1.0, "{seen:?}");
                assert_eq!(seen[4], 1.0);
            }
        }
    }

    /// Only rows added while the list is in view ease in: not the rows it
    /// opened with, nor rows added while it was out of sight.
    #[test]
    fn only_rows_added_in_view_ease_in() {
        let mut arrivals = Arrivals::default();
        arrivals.update(["a", "b"].into_iter(), 1, 0.0);
        assert!(arrivals.since.is_empty(), "the opening rows just show");
        arrivals.update(["a", "b", "c"].into_iter(), 2, 0.05);
        assert_eq!(arrivals.since.keys().collect::<Vec<_>>(), ["c"]);
        arrivals.update(["a", "b", "c"].into_iter(), 3, 0.1);
        assert!(arrivals.since.contains_key("c"), "still easing in");
        arrivals.update(["a", "b", "c"].into_iter(), 4, 1.0);
        assert!(arrivals.since.is_empty(), "settled");
        arrivals.update(["a", "b", "c", "d"].into_iter(), 40, 2.0);
        assert!(arrivals.since.is_empty(), "added while out of sight");
        arrivals.update(["b", "c", "d", "e"].into_iter(), 41, 2.01);
        assert_eq!(arrivals.since.keys().collect::<Vec<_>>(), ["e"]);
    }

    #[test]
    fn a_new_row_eases_in_unless_motion_is_reduced() {
        for reduced in [false, true] {
            let ctx = Context::default();
            let id = Id::new("queue");
            let mut seen = Vec::new();
            frame(&ctx, 0.0, reduced, |ctx| {
                note_rows(ctx, id, ["a"].into_iter())
            });
            for time in [0.02, 0.1, 0.5] {
                frame(&ctx, time, reduced, |ctx| {
                    note_rows(ctx, id, ["a", "b"].into_iter());
                    seen.push(arrival(ctx, id, "b"));
                    assert_eq!(arrival(ctx, id, "a"), 1.0);
                });
            }
            if reduced {
                assert_eq!(seen, [1.0, 1.0, 1.0]);
            } else {
                assert!(seen[0] < seen[1] && seen[1] < 1.0, "{seen:?}");
                assert_eq!(seen[2], 1.0);
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
