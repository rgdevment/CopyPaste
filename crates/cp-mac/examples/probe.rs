#[cfg(target_os = "macos")]
include!("probe/battery.rs");

#[cfg(not(target_os = "macos"))]
fn main() {}
