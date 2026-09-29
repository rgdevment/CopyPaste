use objc2_foundation::{NSString, NSUserDefaults};

const WHAT_IT_SAYS: &str = "AppleInterfaceStyle";

pub fn wants_light() -> Option<bool> {
    let key = NSString::from_str(WHAT_IT_SAYS);
    let said = NSUserDefaults::standardUserDefaults().stringForKey(&key);
    Some(said.is_none_or(|style| !style.to_string().eq_ignore_ascii_case("dark")))
}

#[cfg(test)]
#[path = "theme_test.rs"]
mod tests;
