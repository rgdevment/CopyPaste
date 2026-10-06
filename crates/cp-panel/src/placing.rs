use crate::Panel;
use slint::ComponentHandle;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

pub const NEXT_FRAME: Duration = Duration::from_millis(16);
const OUT: Duration = Duration::from_millis(130);
const GAP: f64 = 12.0;
const EDGE: f64 = 8.0;

pub fn beside(pointer: (f64, f64), area: Area, size: (f64, f64), unit: f64) -> (f64, f64) {
    let (width, height) = size;
    let (gap, edge) = (GAP * unit, EDGE * unit);
    let x = if pointer.0 + gap + width <= area.right {
        pointer.0 + gap
    } else if pointer.0 - gap - width >= area.left {
        pointer.0 - gap - width
    } else {
        area.right - width - gap
    };
    let y = (pointer.1 - height / 2.0)
        .max(area.top + edge)
        .min(area.bottom - edge - height);
    (
        x.min(area.right - width).max(area.left),
        y.min(area.bottom - height).max(area.top),
    )
}

thread_local! {
    static CURTAIN: slint::Timer = slint::Timer::default();
}

pub fn place(panel: &Panel) {
    let Some(pointer) = crate::here::pointer() else {
        return;
    };
    let window = panel.window();
    let scale = f64::from(window.scale_factor());
    let size = window.size();
    let (width, height) = (f64::from(size.width), f64::from(size.height));
    if pointer.physical {
        let (x, y) = beside(pointer.at, pointer.area, (width, height), scale);
        #[allow(clippy::cast_possible_truncation)]
        window.set_position(slint::PhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        ));
    } else {
        let (x, y) = beside(
            pointer.at,
            pointer.area,
            (width / scale, height / scale),
            1.0,
        );
        #[allow(clippy::cast_possible_truncation)]
        window.set_position(slint::LogicalPosition::new(x as f32, y as f32));
    }
}

pub fn nudge(panel: &Panel, dx: f32, dy: f32) {
    let window = panel.window();
    let at = window.position().to_logical(window.scale_factor());
    window.set_position(slint::LogicalPosition::new(at.x + dx, at.y + dy));
}

pub fn appear(ui: &Panel) {
    crate::note::tell("shown");
    ui.set_shown(0.0);
    let weak = ui.as_weak();
    CURTAIN.with(|timer| {
        timer.stop();
        timer.start(slint::TimerMode::SingleShot, NEXT_FRAME, move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_shown(1.0);
            }
        });
    });
}

pub fn vanish(ui: &Panel) {
    ui.set_sheet_open(false);
    ui.set_shown(0.0);
    let weak = ui.as_weak();
    CURTAIN.with(|timer| {
        timer.stop();
        timer.start(slint::TimerMode::SingleShot, OUT, move || {
            if let Some(ui) = weak.upgrade() {
                let _ = ui.hide();
            }
        });
    });
}

#[cfg(test)]
#[path = "placing_test.rs"]
mod tests;
