#[cfg(target_os = "macos")]
include!("probe/battery.rs");

#[cfg(target_os = "macos")]
include!("probe/panel.rs");

#[cfg(not(target_os = "macos"))]
fn main() {}
