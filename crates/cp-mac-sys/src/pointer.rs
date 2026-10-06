use objc2_app_kit::{NSEvent, NSScreen};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    pub x: f64,
    pub y: f64,
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

pub fn spot() -> Option<Spot> {
    let mtm = MainThreadMarker::new()?;
    let at = NSEvent::mouseLocation();
    let screens = NSScreen::screens(mtm);
    let primary = screens.firstObject()?.frame();
    let under = screens
        .iter()
        .find(|screen| holds(screen.frame(), at))
        .or_else(|| NSScreen::mainScreen(mtm))?;
    Some(flipped(at, under.visibleFrame(), primary.size.height))
}

fn holds(frame: NSRect, at: NSPoint) -> bool {
    at.x >= frame.origin.x
        && at.x <= frame.origin.x + frame.size.width
        && at.y >= frame.origin.y
        && at.y <= frame.origin.y + frame.size.height
}

pub fn flipped(at: NSPoint, visible: NSRect, primary_height: f64) -> Spot {
    Spot {
        x: at.x,
        y: primary_height - at.y,
        left: visible.origin.x,
        top: primary_height - (visible.origin.y + visible.size.height),
        right: visible.origin.x + visible.size.width,
        bottom: primary_height - visible.origin.y,
    }
}

#[cfg(test)]
#[path = "pointer_test.rs"]
mod tests;
