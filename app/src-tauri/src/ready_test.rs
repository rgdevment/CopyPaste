use super::corner;
use tauri::{LogicalPosition, PhysicalPosition, PhysicalRect, PhysicalSize};

fn screen(x: i32, y: i32, width: u32, height: u32) -> PhysicalRect<i32, u32> {
    PhysicalRect {
        position: PhysicalPosition::new(x, y),
        size: PhysicalSize::new(width, height),
    }
}

#[test]
fn on_windows_it_sits_over_the_tray() {
    let at = corner(screen(0, 0, 1920, 1032), 1.0, false);
    assert_eq!(at, LogicalPosition::new(1548.0, 836.0));
}

#[test]
fn on_a_mac_it_hangs_under_the_menu_bar() {
    let at = corner(screen(0, 25, 1440, 875), 1.0, true);
    assert_eq!(at, LogicalPosition::new(1068.0, 37.0));
}

#[test]
fn a_taskbar_on_top_still_leaves_it_inside_the_free_space() {
    let at = corner(screen(0, 48, 1920, 1032), 1.0, false);
    assert_eq!(at.y + 184.0 + 12.0, 48.0 + 1032.0);
}

#[test]
fn a_scaled_screen_keeps_the_same_logical_corner() {
    let at = corner(screen(0, 0, 3840, 2064), 2.0, false);
    assert_eq!(at, LogicalPosition::new(1548.0, 836.0));
}
