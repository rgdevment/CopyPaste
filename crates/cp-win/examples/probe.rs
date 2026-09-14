#[cfg(target_os = "windows")]
include!("probe/battery.rs");

#[cfg(not(target_os = "windows"))]
fn main() {}
