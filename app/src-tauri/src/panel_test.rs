use super::*;

#[test]
fn a_sidecar_starts_with_nobody_on_the_other_end() {
    let sidecar = Sidecar::default();
    assert!(sidecar.0.lock().expect("unpoisoned").is_none());
}
