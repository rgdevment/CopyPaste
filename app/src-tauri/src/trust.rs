#[derive(serde::Serialize, PartialEq, Eq, Debug)]
pub struct Trust {
    pub offered: bool,
    pub pastes: bool,
    pub asked_before: bool,
    pub secure_input: bool,
}

#[tauri::command]
pub fn trust() -> Trust {
    there::trust()
}

#[tauri::command]
pub fn ask_trust() -> Trust {
    there::ask();
    there::trust()
}

#[cfg(target_os = "macos")]
mod there {
    use super::Trust;
    use cp_mac_sys::permissions::Readiness;

    pub fn trust() -> Trust {
        let ready = Readiness::probe();
        Trust {
            offered: true,
            pastes: ready.can_paste(),
            asked_before: ready.accessibility,
            secure_input: ready.secure_input,
        }
    }

    pub fn ask() {
        if cp_mac_sys::permissions::can_post_events() {
            return;
        }
        if !cp_mac_sys::permissions::request_post_events() {
            cp_mac_sys::permissions::request_accessibility();
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod there {
    use super::Trust;

    pub fn trust() -> Trust {
        Trust {
            offered: false,
            pastes: false,
            asked_before: false,
            secure_input: false,
        }
    }

    pub fn ask() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn where_nothing_is_asked_nothing_is_offered() {
        let said = trust();
        if !said.offered {
            assert!(!said.pastes && !said.asked_before && !said.secure_input);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn on_a_mac_the_question_always_has_an_answer() {
        assert!(trust().offered);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn pasting_is_never_claimed_without_one_of_the_two_permissions() {
        let said = trust();
        let ready = cp_mac_sys::permissions::Readiness::probe();
        assert_eq!(said.pastes, ready.can_post || ready.accessibility);
    }
}
