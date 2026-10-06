use objc2_app_kit::{NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication};
use objc2_foundation::{MainThreadMarker, NSArray, NSString, NSUserDefaults};

const WHAT_IT_SAYS: &str = "AppleInterfaceStyle";

pub fn wants_light() -> Option<bool> {
    Some(seen_by_the_app().unwrap_or_else(said_by_the_defaults))
}

fn seen_by_the_app() -> Option<bool> {
    let mtm = MainThreadMarker::new()?;
    let (light, dark) = unsafe { (NSAppearanceNameAqua, NSAppearanceNameDarkAqua) };
    let names = NSArray::from_slice(&[light, dark]);
    let best = NSApplication::sharedApplication(mtm)
        .effectiveAppearance()
        .bestMatchFromAppearancesWithNames(&names)?;
    Some(!best.isEqualToString(dark))
}

fn said_by_the_defaults() -> bool {
    let key = NSString::from_str(WHAT_IT_SAYS);
    let said = NSUserDefaults::standardUserDefaults().stringForKey(&key);
    said.is_none_or(|style| !style.to_string().eq_ignore_ascii_case("dark"))
}

#[cfg(test)]
#[path = "theme_test.rs"]
mod tests;
