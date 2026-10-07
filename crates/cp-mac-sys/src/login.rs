use objc2_service_management::{SMAppService, SMAppServiceStatus};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LoginState {
    On,
    Off,
    NeedsApproval,
    Missing,
}

pub fn state_of(raw: isize) -> LoginState {
    match raw {
        1 => LoginState::On,
        2 => LoginState::NeedsApproval,
        3 => LoginState::Missing,
        _ => LoginState::Off,
    }
}

pub fn state() -> LoginState {
    let service = unsafe { SMAppService::mainAppService() };
    let SMAppServiceStatus(raw) = unsafe { service.status() };
    state_of(raw)
}

pub fn set(wanted: bool) -> Result<(), String> {
    let service = unsafe { SMAppService::mainAppService() };
    let done = if wanted {
        unsafe { service.registerAndReturnError() }
    } else {
        unsafe { service.unregisterAndReturnError() }
    };
    done.map_err(|why| why.localizedDescription().to_string())
}

#[cfg(test)]
#[path = "login_test.rs"]
mod tests;
