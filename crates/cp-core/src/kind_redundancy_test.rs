use super::*;
use proptest::prelude::*;

fn with_the_old_guards(text: &str) -> bool {
    let Some((user, host)) = text.split_once('@') else {
        return false;
    };
    if user.is_empty() || host.len() < 3 || host.contains('@') {
        return false;
    }
    is_email(text)
}

proptest! {
    #[test]
    fn the_guards_that_were_here_were_redundant(text in ".{0,40}") {
        prop_assert_eq!(is_email(&text), with_the_old_guards(&text));
    }

    #[test]
    fn the_guards_were_redundant_for_addresses_too(
        user in "[a-z@._%+-]{0,8}",
        host in "[a-z@.-]{0,8}",
    ) {
        let text = format!("{user}@{host}");
        prop_assert_eq!(is_email(&text), with_the_old_guards(&text));
    }
}
