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
