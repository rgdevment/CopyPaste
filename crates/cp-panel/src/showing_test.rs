use super::*;

#[test]
fn a_slow_double_click_lingers_no_more_than_half_a_second() {
    assert_eq!(
        lingering_for(Duration::from_millis(900)),
        Duration::from_millis(500)
    );
}

#[test]
fn a_quick_double_click_lingers_as_long_as_it_takes() {
    assert_eq!(
        lingering_for(Duration::from_millis(300)),
        Duration::from_millis(300)
    );
    assert_eq!(lingering_for(Duration::ZERO), Duration::ZERO);
}

fn once_on_a_testing_platform() {
    thread_local! {
        static READY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    if !READY.replace(true) {
        i_slint_backend_testing::init_no_event_loop();
    }
}

fn look_with(
    size: cp_config::PanelSize,
    density: cp_config::Density,
    text: cp_config::TextSize,
    contrast: bool,
) -> cp_config::Config {
    cp_config::Config {
        panel_size: size,
        density,
        text_size: text,
        high_contrast: contrast,
        ..cp_config::Config::default()
    }
}

struct Opening {
    panel: Panel,
    shelf: crate::kept::Shelf,
    dir: tempfile::TempDir,
}

fn opening(first: &cp_config::Config) -> Opening {
    once_on_a_testing_platform();
    let dir = tempfile::tempdir().expect("dir");
    let fonts = cp_config::look::choices_here();
    let shelf = crate::kept::Shelf::new(crate::kept::resolve(None, &fonts), fonts);
    let panel = Panel::new().expect("the panel builds");
    let held = shelf.clone();
    let weak = panel.as_weak();
    panel.on_fresh_start(move || {
        let panel = weak.unwrap();
        crate::app::dress_theme(panel.global::<crate::Theme>(), held.get());
    });
    let opening = Opening { panel, shelf, dir };
    opening.change_to(first);
    ready(&opening.panel);
    opening
}

impl Opening {
    fn change_to(&self, look: &cp_config::Config) {
        cp_config::write(&cp_config::at(self.dir.path()), look).expect("written");
        self.shelf.renew_from(Some(self.dir.path()));
    }

    fn theme(&self) -> crate::Theme<'_> {
        self.panel.global::<crate::Theme>()
    }

    fn look(&self) -> (f32, f32, bool, bool, f32, f32, f32) {
        let theme = self.theme();
        (
            theme.get_wanted_width(),
            theme.get_wanted_height(),
            theme.get_listing(),
            theme.get_contrast(),
            theme.get_zoom(),
            theme.get_width(),
            theme.get_height(),
        )
    }
}

fn old_look() -> cp_config::Config {
    look_with(
        cp_config::PanelSize::Normal,
        cp_config::Density::Normal,
        cp_config::TextSize::Normal,
        false,
    )
}

fn new_look() -> cp_config::Config {
    look_with(
        cp_config::PanelSize::Tall,
        cp_config::Density::List,
        cp_config::TextSize::Larger,
        true,
    )
}

fn fitted_for(theme: &crate::Theme<'_>) -> Option<(f32, f32)> {
    crate::here::pointer().map(|screen| {
        crate::sizing::fitted(
            (theme.get_wanted_width(), theme.get_wanted_height()),
            theme.get_zoom(),
            crate::sizing::room_of(screen.area, screen.scale),
        )
    })
}

#[test]
fn getting_ready_holds_the_new_look_before_the_window_is_shown() {
    let opening = opening(&old_look());
    assert!(!opening.panel.window().is_visible());
    opening.change_to(&new_look());
    ready(&opening.panel);
    assert!(!opening.panel.window().is_visible());
    let theme = opening.theme();
    let (width, height) = cp_config::PanelSize::Tall.wanted();
    assert_eq!(theme.get_wanted_width(), width);
    assert_eq!(theme.get_wanted_height(), height.unwrap_or(0.0));
    assert!(theme.get_listing());
    assert!(theme.get_contrast());
    assert_eq!(theme.get_zoom(), cp_config::TextSize::Larger.zoom());
    assert_eq!(theme.get_row_listed(), 32.0);
    assert_eq!(theme.get_row_gap(), 2.0);
}

#[test]
fn getting_ready_fits_the_panel_to_the_new_look_before_the_window_is_shown() {
    let opening = opening(&old_look());
    opening.change_to(&new_look());
    ready(&opening.panel);
    let theme = opening.theme();
    if let Some((width, height)) = fitted_for(&theme) {
        assert_eq!((theme.get_width(), theme.get_height()), (width, height));
        assert_ne!(
            height,
            cp_config::PanelSize::Normal.wanted().1.unwrap_or(0.0),
            "a tall panel no longer keeps the old height"
        );
    }
}

#[test]
fn getting_ready_again_with_the_same_look_changes_nothing() {
    let opening = opening(&old_look());
    opening.change_to(&new_look());
    ready(&opening.panel);
    let once = opening.look();
    ready(&opening.panel);
    assert_eq!(opening.look(), once);
}

#[test]
fn getting_ready_applies_a_change_of_density_alone() {
    let opening = opening(&old_look());
    assert!(!opening.theme().get_listing());
    let before = opening.look();
    opening.change_to(&look_with(
        cp_config::PanelSize::Normal,
        cp_config::Density::List,
        cp_config::TextSize::Normal,
        false,
    ));
    ready(&opening.panel);
    let after = opening.look();
    assert!(opening.theme().get_listing());
    assert_eq!(opening.theme().get_row_gap(), 2.0);
    assert_eq!((after.0, after.1, after.4), (before.0, before.1, before.4));
    assert_eq!((after.5, after.6), (before.5, before.6));
}
