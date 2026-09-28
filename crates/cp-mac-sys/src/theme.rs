use objc2_foundation::{NSString, NSUserDefaults};

const WHAT_IT_SAYS: &str = "AppleInterfaceStyle";

pub fn wants_light() -> Option<bool> {
    let key = NSString::from_str(WHAT_IT_SAYS);
    let said = NSUserDefaults::standardUserDefaults().stringForKey(&key);
    Some(said.is_none_or(|style| !style.to_string().eq_ignore_ascii_case("dark")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_always_has_an_opinion_about_its_own_theme() {
        assert!(wants_light().is_some());
    }

    #[test]
    fn asking_twice_gives_the_same_answer() {
        assert_eq!(wants_light(), wants_light());
    }
}
