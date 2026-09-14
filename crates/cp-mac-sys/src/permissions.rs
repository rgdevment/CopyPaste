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
        self.can_post
    }

    pub fn can_use_menu_fallback(&self) -> bool {
        self.accessibility
    }
}

#[cfg(test)]
mod tests {
    use super::Readiness;

    fn with(can_post: bool, accessibility: bool, secure_input: bool) -> Readiness {
        Readiness {
            can_post,
            accessibility,
            secure_input,
        }
    }

    #[test]
    fn pasting_depends_on_posting_events_and_nothing_else() {
        assert!(with(true, false, false).can_paste());
        assert!(
            with(true, false, true).can_paste(),
            "el input seguro no manda"
        );
        assert!(with(true, true, true).can_paste());
        assert!(!with(false, true, false).can_paste());
    }

    #[test]
    fn the_menu_fallback_needs_its_own_permission() {
        assert!(!with(true, false, false).can_use_menu_fallback());
        assert!(with(true, true, false).can_use_menu_fallback());
    }

    #[test]
    fn the_two_permissions_are_independent() {
        assert!(with(true, false, false).can_paste());
        assert!(!with(true, false, false).can_use_menu_fallback());
        assert!(!with(false, true, false).can_paste());
        assert!(with(false, true, false).can_use_menu_fallback());
    }
}
