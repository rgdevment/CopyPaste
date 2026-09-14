//! Batería de pruebas del núcleo contra macOS de verdad.
//!
//! No son pruebas de `cargo test` a propósito: `NSPasteboard` exige el hilo
//! principal y el runner no lo garantiza, y varias tocan aplicaciones reales.
//! Se ejecuta con `cargo run -p cp-mac --example probe`.

// El cuerpo vive aparte porque un `use` de objc2 no se resuelve fuera de Apple
// y `cargo test --workspace` alcanza los ejemplos. Cargo solo autodescubre
// `examples/*.rs` y `examples/*/main.rs`, así que este no se compila solo.
#[cfg(target_os = "macos")]
include!("probe/battery.rs");

#[cfg(not(target_os = "macos"))]
fn main() {}
