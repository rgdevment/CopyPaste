use super::*;

fn screenshot(shared_header: usize, tail: u8) -> Vec<u8> {
    let mut one = vec![0xAB; shared_header];
    one.resize(1024 * 1024, tail);
    one
}

#[test]
fn two_captures_sharing_a_menu_bar_are_not_the_same_capture() {
    let a = screenshot(512 * 1024, 0x01);
    let b = screenshot(512 * 1024, 0x02);
    assert_eq!(a.len(), b.len(), "same size, half the image identical");
    assert_ne!(
        content_hash(&a),
        content_hash(&b),
        "sampling just the header would call them equal"
    );
}

#[test]
fn two_long_texts_that_begin_alike_are_different_items() {
    let shared = "x".repeat(100);
    let a = format!("{shared}first");
    let b = format!("{shared}second");
    assert_ne!(content_hash(a.as_bytes()), content_hash(b.as_bytes()));
}

#[test]
fn nothing_at_all_still_has_an_identity() {
    assert_eq!(content_hash(&[]), content_hash(&[]));
    assert_ne!(content_hash(&[]), content_hash(&[0]));
    assert_ne!(content_hash(&[0]), content_hash(&[1]));
}

#[test]
fn the_same_content_always_hashes_the_same() {
    let big = screenshot(512 * 1024, 0x07);
    assert_eq!(content_hash(&big), content_hash(&big.clone()));
}

#[test]
fn the_sampling_reaches_the_end_of_the_buffer() {
    let big = vec![0x11; 4 * 1024 * 1024];
    let mut tail_changed = big.clone();
    *tail_changed.last_mut().unwrap() = 0x22;
    assert_ne!(
        content_hash(&big),
        content_hash(&tail_changed),
        "if the blocks all fell at the start, this would go unnoticed"
    );
}

#[test]
fn a_change_between_blocks_is_invisible_and_that_is_the_deal() {
    let big = vec![0x11; 4 * 1024 * 1024];
    let mut hole = big.clone();
    hole[2 * 1024 * 1024] = 0x22;
    assert_eq!(
        content_hash(&big),
        content_hash(&hole),
        "if this changed, the sampling would have stopped being a sampling"
    );
}

#[test]
fn the_two_paths_meet_at_the_threshold() {
    let under = vec![0x33; WHOLE_UP_TO];
    let over = vec![0x33; WHOLE_UP_TO + 1];
    assert_eq!(
        content_hash(&under),
        xxh3_64(&under),
        "up to the threshold, the whole content is looked at"
    );
    assert_ne!(
        content_hash(&over),
        xxh3_64(&over),
        "above it, it is sampled, so it does not match the direct hash"
    );
}

#[test]
fn each_of_the_sixteen_blocks_is_looked_at() {
    let size = 4 * 1024 * 1024;
    let base = vec![0x44; size];
    let reference = content_hash(&base);
    for block in 0..BLOCKS {
        let mut poked = base.clone();
        let at = (size - BLOCK) * block / (BLOCKS - 1);
        poked[at] = 0x55;
        assert_ne!(
            content_hash(&poked),
            reference,
            "block {block}, starting at {at}, is missing from the mix"
        );
    }
}

#[test]
fn length_alone_separates_two_otherwise_identical_samples() {
    let a = vec![0x5A; 2 * 1024 * 1024];
    let mut b = a.clone();
    b.push(0x5A);
    assert_ne!(content_hash(&a), content_hash(&b));
}
