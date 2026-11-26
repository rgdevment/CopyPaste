const PENDING: &str = "PENDING_";

fn main() {
    for (what, said) in [
        ("tauri.conf.json", include_str!("tauri.conf.json")),
        ("updater-spare.pub", include_str!("updater-spare.pub")),
    ] {
        assert!(
            !said.contains(PENDING),
            "{what} still holds a placeholder where an updater public key goes"
        );
    }
    tauri_build::build();
}
