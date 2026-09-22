use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DWM_SYSTEMBACKDROP_TYPE, DWMSBT_MAINWINDOW, DWMSBT_NONE, DWMSBT_TRANSIENTWINDOW,
    DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute,
};
use windows::core::BOOL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backdrop {
    Mica,
    Acrylic,
    None,
}

impl Backdrop {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "mica" => Some(Backdrop::Mica),
            "acrylic" => Some(Backdrop::Acrylic),
            "none" => Some(Backdrop::None),
            _ => None,
        }
    }

    fn system_type(self) -> DWM_SYSTEMBACKDROP_TYPE {
        match self {
            Backdrop::Mica => DWMSBT_MAINWINDOW,
            Backdrop::Acrylic => DWMSBT_TRANSIENTWINDOW,
            Backdrop::None => DWMSBT_NONE,
        }
    }
}

pub fn apply(window: isize, backdrop: Backdrop, dark: bool) -> bool {
    let hwnd = HWND(window as *mut std::ffi::c_void);
    let immersive = BOOL(i32::from(dark));

    let themed = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&immersive as *const BOOL).cast(),
            std::mem::size_of::<BOOL>() as u32,
        )
    };
    let kind = backdrop.system_type();

    let backed = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            (&kind as *const DWM_SYSTEMBACKDROP_TYPE).cast(),
            std::mem::size_of::<DWM_SYSTEMBACKDROP_TYPE>() as u32,
        )
    };
    themed.is_ok() && backed.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_a_setting_would_use_map_to_a_backdrop() {
        assert_eq!(Backdrop::from_name("mica"), Some(Backdrop::Mica));
        assert_eq!(Backdrop::from_name("acrylic"), Some(Backdrop::Acrylic));
        assert_eq!(Backdrop::from_name("none"), Some(Backdrop::None));
        assert_eq!(Backdrop::from_name("blur"), None);
    }

    #[test]
    fn a_window_that_does_not_exist_is_refused_not_crashed_on() {
        assert!(!apply(0, Backdrop::Mica, true));
        assert!(!apply(0x7fff_ffff, Backdrop::None, false));
    }
}
