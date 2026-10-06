use crate::Panel;
use slint::ComponentHandle;
use std::time::Duration;

pub const NEXT_FRAME: Duration = Duration::from_millis(16);
const OUT: Duration = if cfg!(target_os = "macos") {
    Duration::ZERO
} else {
    Duration::from_millis(130)
};
pub const SLOW: Duration = Duration::from_millis(100);

thread_local! {
    static CURTAIN: slint::Timer = slint::Timer::default();
}

pub fn place(panel: &Panel) {
    let Some(pointer) = crate::here::pointer() else {
        return;
    };
    let theme = panel.global::<crate::Theme>();
    let width = f64::from(theme.get_width() + theme.get_margin() * 2.0);
    let height = f64::from(theme.get_height() + theme.get_margin() * 2.0);
    let window = panel.window();
    let scale = pointer.scale;
    if pointer.physical {
        let (x, y) = crate::placing::beside(
            pointer.at,
            pointer.area,
            (width * scale, height * scale),
            scale,
        );
        #[allow(clippy::cast_possible_truncation)]
        window.set_position(slint::PhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        ));
    } else {
        let (x, y) = crate::placing::beside(pointer.at, pointer.area, (width, height), 1.0);
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
        if OUT.is_zero() {
            let _ = ui.hide();
            return;
        }
        timer.start(slint::TimerMode::SingleShot, OUT, move || {
            if let Some(ui) = weak.upgrade() {
                let _ = ui.hide();
            }
        });
    });
}

pub fn leave_when_left(panel: &Panel, hides: fn() -> bool) {
    use i_slint_backend_winit::winit::event::WindowEvent;
    use i_slint_backend_winit::{EventResult, WinitWindowAccessor};
    let weak = panel.as_weak();
    panel.window().on_winit_window_event(move |_, event| {
        if let WindowEvent::Focused(false) = event
            && let Some(ui) = weak.upgrade()
            && ui.window().is_visible()
            && hides()
        {
            let later = ui.as_weak();
            slint::Timer::single_shot(Duration::ZERO, move || {
                if let Some(ui) = later.upgrade() {
                    vanish(&ui);
                }
            });
        }
        EventResult::Propagate
    });
}
