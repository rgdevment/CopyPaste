use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

pub fn as_accessory() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    NSApplication::sharedApplication(mtm)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory)
}

// an accessory app never shows up as NSWorkspace's frontmost one, so «are we in front»
// has to be asked of the application itself
pub fn is_ours_up_front() -> bool {
    MainThreadMarker::new().is_some_and(|mtm| NSApplication::sharedApplication(mtm).isActive())
}

pub fn is_accessory() -> Option<bool> {
    let mtm = MainThreadMarker::new()?;
    Some(
        NSApplication::sharedApplication(mtm).activationPolicy()
            == NSApplicationActivationPolicy::Accessory,
    )
}

#[cfg(test)]
#[path = "activation_test.rs"]
mod tests;
