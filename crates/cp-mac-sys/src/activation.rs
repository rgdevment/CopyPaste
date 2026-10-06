use objc2_app_kit::NSApplication;
use objc2_foundation::MainThreadMarker;

pub fn is_ours_up_front() -> Option<bool> {
    let mtm = MainThreadMarker::new()?;
    let app = NSApplication::sharedApplication(mtm);
    Some(app.isActive() || app.keyWindow().is_some())
}

#[cfg(test)]
#[path = "activation_test.rs"]
mod tests;
