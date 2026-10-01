use super::*;

fn these(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|one| (*one).to_owned()).collect()
}

#[test]
fn a_plain_click_leaves_only_what_was_clicked() {
    assert_eq!(after(&these(&["image"]), "text", false), these(&["text"]));
    assert_eq!(
        after(&these(&["image", "text"]), "json", false),
        these(&["json"])
    );
}

#[test]
fn a_plain_click_never_grows_the_filter() {
    for chosen in [
        these(&[]),
        these(&["image"]),
        these(&["image", "text"]),
        these(&["image", "text", "json"]),
    ] {
        let next = after(&chosen, "link", false);
        assert!(next.len() <= 1, "a click left {next:?}");
    }
}

#[test]
fn clicking_the_only_one_chosen_takes_the_filter_off() {
    assert!(after(&these(&["image"]), "image", false).is_empty());
}

#[test]
fn clicking_one_of_several_keeps_that_one_instead_of_clearing() {
    assert_eq!(
        after(&these(&["image", "text"]), "image", false),
        these(&["image"]),
        "with two chosen, the click means only this one"
    );
}

#[test]
fn adding_keeps_what_was_already_there() {
    assert_eq!(
        after(&these(&["image"]), "text", true),
        these(&["image", "text"])
    );
    assert_eq!(after(&these(&[]), "image", true), these(&["image"]));
}

#[test]
fn adding_one_that_is_already_in_takes_it_out() {
    assert_eq!(
        after(&these(&["image", "text"]), "image", true),
        these(&["text"])
    );
}

#[test]
fn adding_respects_the_order_they_were_chosen_in() {
    assert_eq!(
        after(&these(&["image", "text", "json"]), "link", true),
        these(&["image", "text", "json", "link"])
    );
}

#[test]
fn nothing_is_ever_chosen_twice() {
    let once = after(&these(&["image"]), "image", true);
    assert!(once.is_empty());
    let twice = after(&these(&["image", "text"]), "text", false);
    assert_eq!(twice, these(&["text"]));
}
