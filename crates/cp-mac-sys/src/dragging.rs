use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{AllocAnyThread, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession,
    NSDraggingSource, NSEvent, NSEventModifierFlags, NSEventType, NSImage, NSPasteboardWriting,
    NSView, NSWorkspace,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSObject, NSObjectProtocol, NSPoint, NSProcessInfo, NSRect, NSSize,
    NSString, NSURL,
};
use std::cell::OnceCell;
use std::ffi::c_void;
use std::path::Path;
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dragged {
    Started,
    Nothing,
    NoEvent,
    Elsewhere,
}

pub const ICON_SIDE: f64 = 48.0;

const STACKED_BY: f64 = 6.0;

const _: () = assert!(STACKED_BY * 4.0 < ICON_SIDE);

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CopyPasteDragSource"]
    struct Source;

    unsafe impl NSObjectProtocol for Source {}

    unsafe impl NSDraggingSource for Source {
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn operation_mask(
            &self,
            _session: &NSDraggingSession,
            _context: NSDraggingContext,
        ) -> NSDragOperation {
            NSDragOperation::Copy
        }
    }
);

impl Source {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![Self::alloc(mtm), init] }
    }
}

thread_local! {
    static SOURCE: OnceCell<Retained<Source>> = const { OnceCell::new() };
}

fn in_hand(mtm: MainThreadMarker) -> Option<Retained<NSEvent>> {
    let event = NSApplication::sharedApplication(mtm).currentEvent()?;
    matches!(
        event.r#type(),
        NSEventType::LeftMouseDown | NSEventType::LeftMouseDragged
    )
    .then_some(event)
}

fn described(view: &NSView) -> Option<Retained<NSEvent>> {
    let window = view.window()?;
    let at = window.convertPointFromScreen(NSEvent::mouseLocation());
    NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
        NSEventType::LeftMouseDragged,
        at,
        NSEventModifierFlags::empty(),
        NSProcessInfo::processInfo().systemUptime(),
        window.windowNumber(),
        None,
        0,
        1,
        1.0,
    )
}

fn mouse_event(view: &NSView, mtm: MainThreadMarker) -> Option<Retained<NSEvent>> {
    in_hand(mtm).or_else(|| described(view))
}

fn item_for(path: &Path, at: NSPoint) -> Option<Retained<NSDraggingItem>> {
    let said = NSString::from_str(path.to_str()?);
    let url = NSURL::fileURLWithPath(&said);
    let writer = ProtocolObject::<dyn NSPasteboardWriting>::from_ref(&*url);
    let item = NSDraggingItem::initWithPasteboardWriter(NSDraggingItem::alloc(), writer);
    let icon: Retained<NSImage> = NSWorkspace::sharedWorkspace().iconForFile(&said);
    let frame = NSRect::new(at, NSSize::new(ICON_SIDE, ICON_SIDE));
    unsafe { item.setDraggingFrame_contents(frame, Some(&icon)) };
    Some(item)
}

pub fn from_view(ns_view: NonNull<c_void>, paths: &[&Path]) -> Dragged {
    let Some(mtm) = MainThreadMarker::new() else {
        return Dragged::Elsewhere;
    };
    if paths.is_empty() {
        return Dragged::Nothing;
    }
    let view: &NSView = unsafe { ns_view.cast::<NSView>().as_ref() };
    let Some(event) = mouse_event(view, mtm) else {
        return Dragged::NoEvent;
    };
    let where_it_is = view.convertPoint_fromView(event.locationInWindow(), None);
    let items: Vec<Retained<NSDraggingItem>> = paths
        .iter()
        .enumerate()
        .filter_map(|(nth, path)| {
            let nudge = nth as f64 * STACKED_BY;
            let at = NSPoint::new(
                where_it_is.x - ICON_SIDE / 2.0 + nudge,
                where_it_is.y - ICON_SIDE / 2.0 - nudge,
            );
            item_for(path, at)
        })
        .collect();
    if items.is_empty() {
        return Dragged::Nothing;
    }
    let source = SOURCE.with(|once| once.get_or_init(|| Source::new(mtm)).clone());
    view.beginDraggingSessionWithItems_event_source(
        &NSArray::from_retained_slice(&items),
        &event,
        ProtocolObject::<dyn NSDraggingSource>::from_ref(&*source),
    );
    Dragged::Started
}

#[cfg(test)]
#[path = "dragging_test.rs"]
mod tests;
