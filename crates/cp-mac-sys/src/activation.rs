use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

pub fn as_accessory() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    NSApplication::sharedApplication(mtm)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory)
}

pub fn is_ours_up_front() -> Option<bool> {
    let mtm = MainThreadMarker::new()?;
    Some(NSApplication::sharedApplication(mtm).isActive())
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
