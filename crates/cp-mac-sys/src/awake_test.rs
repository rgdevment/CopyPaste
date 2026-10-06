use super::*;

#[test]
fn an_activity_can_be_begun_and_held_and_dropped() {
    let held = keep_awake("test");
    drop(held);
}
