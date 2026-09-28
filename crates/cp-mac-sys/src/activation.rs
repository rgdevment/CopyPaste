use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

pub fn as_accessory() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    NSApplication::sharedApplication(mtm)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory)
}

pub fn is_accessory() -> Option<bool> {
    let mtm = MainThreadMarker::new()?;
    Some(
        NSApplication::sharedApplication(mtm).activationPolicy()
            == NSApplicationActivationPolicy::Accessory,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panel_that_never_owns_the_dock_can_say_so() {
        if MainThreadMarker::new().is_none() {
            return;
        }
        assert!(as_accessory());
        assert_eq!(is_accessory(), Some(true));
    }
}
