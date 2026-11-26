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
const LINGERS_AT_MOST: Duration = Duration::from_millis(500);

thread_local! {
    static CURTAIN: slint::Timer = slint::Timer::default();
}

fn fit(panel: &Panel, screen: Option<crate::here::Pointer>) {
    let theme = panel.global::<crate::Theme>();
    let Some(screen) = screen else {
        return;
    };
    let (width, height) = crate::sizing::fitted(
        (theme.get_wanted_width(), theme.get_wanted_height()),
        theme.get_zoom(),
        crate::sizing::room_of(screen.area, screen.scale),
    );
    theme.set_width(width);
    theme.set_height(height);
}

pub fn ready(panel: &Panel) {
    crate::view::dress_words(panel);
    panel.invoke_fresh_start();
    resize(panel, crate::here::pointer().or_else(|| own_screen(panel)));
}

pub fn place(panel: &Panel) {
    fit(panel, crate::here::pointer());
    let theme = panel.global::<crate::Theme>();
    let zoom = f64::from(theme.get_zoom());
    let size = (
        f64::from(theme.get_width()) * zoom,
        f64::from(theme.get_height()) * zoom,
    );
    beside_pointer(panel.window(), size);
}

pub fn kept_inside(window: &slint::Window, (width, height): (f64, f64)) {
    let Some(screen) = crate::app::handle_of(window).and_then(crate::here::screen_of) else {
        return;
    };
    let now = window.position();
    let (x, y) = crate::placing::kept_within(
        (f64::from(now.x), f64::from(now.y)),
        (width * screen.scale, height * screen.scale),
        screen.area,
    );
    #[allow(clippy::cast_possible_truncation)]
    window.set_position(slint::PhysicalPosition::new(
        x.round() as i32,
        y.round() as i32,
    ));
}

pub fn beside_pointer(window: &slint::Window, (width, height): (f64, f64)) {
    let Some(pointer) = crate::here::pointer() else {
        return;
    };
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

pub fn rescaled(window: &slint::Window, zoom: f32) -> Option<f32> {
    use i_slint_backend_winit::WinitWindowAccessor;
    let device = window.with_winit_window(|winit| winit.scale_factor())?;
    #[allow(clippy::cast_possible_truncation)]
    let wanted = (device * f64::from(zoom)) as f32;
    if (window.scale_factor() - wanted).abs() < 0.001 {
        return None;
    }
    window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
        scale_factor: wanted,
    });
    Some(wanted)
}

fn own_screen(panel: &Panel) -> Option<crate::here::Pointer> {
    crate::app::handle_of(panel.window()).and_then(crate::here::screen_of)
}

pub fn zoom(panel: &Panel) {
    resize(panel, own_screen(panel).or_else(crate::here::pointer));
}

fn resize(panel: &Panel, screen: Option<crate::here::Pointer>) {
    use i_slint_backend_winit::WinitWindowAccessor;
    fit(panel, screen);
    let window = panel.window();
    let theme = panel.global::<crate::Theme>();
    let wanted = rescaled(window, theme.get_zoom()).unwrap_or_else(|| window.scale_factor());
    let side = |logical: f32| (logical * wanted).round().max(1.0) as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let size = slint::PhysicalSize::new(side(theme.get_width()), side(theme.get_height()));
    let bound = i_slint_backend_winit::winit::dpi::PhysicalSize::new(size.width, size.height);
    window.with_winit_window(|winit| {
        winit.set_min_inner_size(Some(bound));
        winit.set_max_inner_size(Some(bound));
    });
    window.set_size(size);
    let zoom = f64::from(theme.get_zoom());
    kept_inside(
        window,
        (
            f64::from(theme.get_width()) * zoom,
            f64::from(theme.get_height()) * zoom,
        ),
    );
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

pub fn lingering_for(double_click: Duration) -> Duration {
    double_click.min(LINGERS_AT_MOST)
}

pub fn linger(ui: &Panel, timer: &slint::Timer) {
    let lasts = lingering_for(crate::here::double_click());
    ui.set_linger(i64::try_from(lasts.as_millis()).unwrap_or(0));
    ui.set_lingering(true);
    clicks_activate(ui, false);
    let weak = ui.as_weak();
    timer.start(slint::TimerMode::SingleShot, lasts, move || {
        if let Some(ui) = weak.upgrade() {
            let _ = ui.hide();
            settle(&ui);
        }
    });
}

pub fn stop_lingering(ui: &Panel, timer: &slint::Timer) {
    timer.stop();
    settle(ui);
}

fn settle(ui: &Panel) {
    if ui.get_lingering() {
        ui.set_lingering(false);
        clicks_activate(ui, true);
    }
}

fn clicks_activate(ui: &Panel, activate: bool) {
    if let Some(handle) = crate::app::handle_of(ui.window()) {
        crate::here::clicks_activate(handle, activate);
    }
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
            && !ui.get_lingering()
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

#[cfg(test)]
#[path = "showing_test.rs"]
mod tests;
