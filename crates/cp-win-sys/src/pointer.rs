use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

const PLAIN_DPI: f64 = 96.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    pub x: i32,
    pub y: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub scale: f64,
}

pub fn spot() -> Option<Spot> {
    let mut at = POINT::default();
    unsafe { GetCursorPos(&raw mut at) }.ok()?;
    let monitor = unsafe { MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: u32::try_from(size_of::<MONITORINFO>()).ok()?,
        ..Default::default()
    };
    if !unsafe { GetMonitorInfoW(monitor, &raw mut info) }.as_bool() {
        return None;
    }
    let (mut dpi, mut unused) = (0_u32, 0_u32);
    let scale =
        unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &raw mut dpi, &raw mut unused) }
            .ok()
            .filter(|()| dpi > 0)
            .map_or(1.0, |()| f64::from(dpi) / PLAIN_DPI);
    let work = info.rcWork;
    Some(Spot {
        x: at.x,
        y: at.y,
        left: work.left,
        top: work.top,
        right: work.right,
        bottom: work.bottom,
        scale,
    })
}

#[cfg(test)]
#[path = "pointer_test.rs"]
mod tests;
