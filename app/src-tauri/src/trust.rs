#[derive(serde::Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trust {
    pub offered: bool,
    pub pastes: bool,
    pub secure_input: bool,
    pub clipboard: &'static str,
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
    use cp_mac_sys::pasteboard::Access;
    use cp_mac_sys::permissions::Readiness;

    pub fn trust() -> Trust {
        let ready = Readiness::probe();
        Trust {
            offered: true,
            pastes: ready.can_paste(),
            secure_input: ready.secure_input,
            clipboard: reading(cp_mac_sys::pasteboard::access_from_any_thread()),
        }
    }

    pub fn reading(access: Option<Access>) -> &'static str {
        match access {
            Some(Access::AlwaysDeny) => "denied",
            Some(Access::Ask) => "asks",
            _ => "allowed",
        }
    }

    pub fn ask() {
        if !cp_mac_sys::permissions::can_post_events() {
            cp_mac_sys::permissions::request_post_events();
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
            secure_input: false,
            clipboard: "allowed",
        }
    }

    pub fn ask() {}
}

#[cfg(test)]
#[path = "trust_test.rs"]
mod tests;
