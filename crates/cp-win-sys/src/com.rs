use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};

const VERBATIM: &str = r"\\?\";

pub struct Apartment {
    ours: bool,
}

impl Apartment {
    pub fn enter() -> Self {
        let entered =
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
        Self {
            ours: entered.is_ok(),
        }
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.ours {
            unsafe { CoUninitialize() };
        }
    }
}

pub fn shell_path(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let absolute = path.canonicalize().ok()?;
    let text = absolute.to_str()?;
    Some(std::path::PathBuf::from(
        text.strip_prefix(VERBATIM).unwrap_or(text),
    ))
}

#[cfg(test)]
#[path = "com_test.rs"]
mod tests;
