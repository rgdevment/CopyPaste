use super::corner;
use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize};

fn screen(x: i32, y: i32, width: u32, height: u32) -> PhysicalRect<i32, u32> {
    PhysicalRect {
        position: PhysicalPosition::new(x, y),
        size: PhysicalSize::new(width, height),
    }
}

#[test]
fn on_windows_it_sits_over_the_tray() {
    let at = corner(screen(0, 0, 1920, 1032), 1.0, false);
    assert_eq!(at, PhysicalPosition::new(1920 - 360 - 12, 1032 - 184 - 12));
}

#[test]
fn on_a_mac_it_hangs_under_the_menu_bar() {
    let at = corner(screen(0, 25, 1440, 875), 1.0, true);
    assert_eq!(at, PhysicalPosition::new(1440 - 360 - 12, 25 + 12));
}

#[test]
fn a_taskbar_on_top_still_leaves_it_inside_the_free_space() {
    let at = corner(screen(0, 48, 1920, 1032), 1.0, false);
    assert_eq!(at.y + 184 + 12, 48 + 1032);
}

#[test]
fn it_grows_with_the_screen_scale() {
    let at = corner(screen(0, 0, 3840, 2088), 2.0, false);
    assert_eq!(at, PhysicalPosition::new(3840 - 720 - 24, 2088 - 368 - 24));
}
