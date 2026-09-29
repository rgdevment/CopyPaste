use objc2_application_services::{AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt};
use objc2_core_foundation::{CFBoolean, CFDictionary};

unsafe extern "C" {
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
    fn AXIsProcessTrusted() -> bool;
    fn IsSecureEventInputEnabled() -> bool;
}

pub fn can_post_events() -> bool {
    unsafe { CGPreflightPostEventAccess() }
}

pub fn request_post_events() -> bool {
    unsafe { CGRequestPostEventAccess() }
}

pub fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn request_accessibility() -> bool {
    let prompt = unsafe { kAXTrustedCheckOptionPrompt };
    let options = CFDictionary::from_slices(&[prompt], &[CFBoolean::new(true)]);
    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) }
}

pub fn is_secure_input_enabled() -> bool {
    unsafe { IsSecureEventInputEnabled() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Readiness {
    pub can_post: bool,
    pub accessibility: bool,
    pub secure_input: bool,
}

impl Readiness {
    pub fn probe() -> Self {
        Self {
            can_post: can_post_events(),
            accessibility: is_accessibility_trusted(),
            secure_input: is_secure_input_enabled(),
        }
    }

    pub fn can_paste(&self) -> bool {
        self.can_post || self.accessibility
    }

    pub fn can_use_menu_fallback(&self) -> bool {
        self.accessibility
    }
}

#[cfg(test)]
#[path = "permissions_test.rs"]
mod tests;
