use windows::Services::Store::StoreContext;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::CoIncrementMTAUsage;
use windows::Win32::System::Recovery::{
    RESTART_NO_CRASH, RESTART_NO_HANG, RESTART_NO_REBOOT, RegisterApplicationRestart,
};
use windows::Win32::UI::Shell::IInitializeWithWindow;
use windows::core::{HSTRING, Interface};

pub fn reopen_when_replaced() -> windows::core::Result<()> {
    let same_line = HSTRING::new();
    unsafe {
        RegisterApplicationRestart(
            &same_line,
            RESTART_NO_CRASH | RESTART_NO_HANG | RESTART_NO_REBOOT,
        )
    }
}

pub fn store_context(window: isize) -> windows::core::Result<StoreContext> {
    join_the_apartment();
    let shop = StoreContext::GetDefault()?;
    let owner: IInitializeWithWindow = shop.cast()?;
    unsafe { owner.Initialize(HWND(window as *mut core::ffi::c_void)) }?;
    Ok(shop)
}

fn join_the_apartment() {
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        let _ = unsafe { CoIncrementMTAUsage() };
    });
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
