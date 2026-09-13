use objc2_app_kit::NSPasteboard;
use objc2_foundation::MainThreadMarker;

/// `NSPasteboard` tiene un fallo de concurrencia con file promises, así que solo
/// se toca desde el hilo principal: el `MainThreadMarker` lo obliga en compilación.
pub fn change_count(_mtm: MainThreadMarker) -> isize {
    NSPasteboard::generalPasteboard().changeCount()
}
