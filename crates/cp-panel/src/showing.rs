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
    let zoom = f64::from(theme.get_zoom());
    let width = f64::from(theme.get_width() + theme.get_margin() * 2.0) * zoom;
    let height = f64::from(theme.get_height() + theme.get_margin() * 2.0) * zoom;
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

pub fn zoom(panel: &Panel) {
    use i_slint_backend_winit::WinitWindowAccessor;
    let window = panel.window();
    let Some(device) = window.with_winit_window(|winit| winit.scale_factor()) else {
        return;
    };
    let theme = panel.global::<crate::Theme>();
    #[allow(clippy::cast_possible_truncation)]
    let wanted = (device * f64::from(theme.get_zoom())) as f32;
    if (window.scale_factor() - wanted).abs() < 0.001 {
        return;
    }
    window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
        scale_factor: wanted,
    });
    let side = |logical: f32| (logical * wanted).round().max(1.0) as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let size = slint::PhysicalSize::new(
        side(theme.get_width() + theme.get_margin() * 2.0),
        side(theme.get_height() + theme.get_margin() * 2.0),
    );
    let bound = i_slint_backend_winit::winit::dpi::PhysicalSize::new(size.width, size.height);
    window.with_winit_window(|winit| {
        winit.set_min_inner_size(Some(bound));
        winit.set_max_inner_size(Some(bound));
    });
    window.set_size(size);
}

pub fn nudge(panel: &Panel, dx: f32, dy: f32) {
    let window = panel.window();
    let at = window.position();
    let scale = window.scale_factor();
    #[allow(clippy::cast_possible_truncation)]
    window.set_position(slint::PhysicalPosition::new(
        at.x + (dx * scale).round() as i32,
        at.y + (dy * scale).round() as i32,
    ));
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

pub fn leave_when_left(panel: &Panel, hides: impl Fn() -> bool + 'static) {
    use i_slint_backend_winit::winit::event::WindowEvent;
    use i_slint_backend_winit::{EventResult, WinitWindowAccessor};
    let weak = panel.as_weak();
    panel.window().on_winit_window_event(move |_, event| {
        if let WindowEvent::ScaleFactorChanged { .. } = event {
            let later = weak.clone();
            slint::Timer::single_shot(Duration::ZERO, move || {
                if let Some(ui) = later.upgrade() {
                    zoom(&ui);
                }
            });
        }
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
