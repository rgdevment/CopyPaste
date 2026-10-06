use objc2::rc::Retained;
use objc2_foundation::{NSActivityOptions, NSObjectProtocol, NSProcessInfo, NSString};

pub struct Awake {
    _token: Retained<objc2::runtime::ProtocolObject<dyn NSObjectProtocol>>,
}

pub fn keep_awake(reason: &str) -> Awake {
    let options = NSActivityOptions::UserInitiatedAllowingIdleSystemSleep;
    let token = NSProcessInfo::processInfo()
        .beginActivityWithOptions_reason(options, &NSString::from_str(reason));
    Awake { _token: token }
}

#[cfg(test)]
#[path = "awake_test.rs"]
mod tests;
