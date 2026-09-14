fn main() {
    // Sin la guarda, `cargo test --workspace` en Windows falla antes de
    // compilar una sola linea del nucleo: los frameworks solo existen en Apple.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    println!("cargo:rustc-link-lib=framework=Carbon");
}
