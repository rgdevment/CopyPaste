use super::*;

#[test]
fn the_process_asks_to_come_back_after_it_is_replaced() {
    assert!(reopen_when_replaced().is_ok());
}

#[test]
fn a_store_context_is_handed_out_with_its_owner_window_set() {
    assert!(store_context(0).is_ok());
}
