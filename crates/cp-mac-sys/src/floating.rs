use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{ClassType, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSBackingStoreType, NSPanel, NSResponder, NSView, NSWindow, NSWindowAnimationBehavior,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSKeyValueObservingOptions, NSObject, NSObjectNSKeyValueObserverRegistration,
    NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
};
use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::NonNull;

define_class!(
    #[unsafe(super(NSPanel, NSWindow, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CopyPastePanel"]
    struct Floating;

    impl Floating {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> bool {
            true
        }
    }
);

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CopyPastePanelLooker"]
    struct Looker;

    impl Looker {
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        fn looked(
            &self,
            _path: Option<&NSString>,
            _of: Option<&AnyObject>,
            _change: Option<&AnyObject>,
            _context: *mut c_void,
        ) {
        }
    }
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Floated {
    pub became_a_panel: bool,
    pub never_activates: bool,
    pub joins_full_screen: bool,
    pub follows_the_space: bool,
    pub stays_when_left: bool,
    pub appears_at_once: bool,
    pub took_the_keys: bool,
    pub counts_as_ours: bool,
    pub back_as_it_was: bool,
}

thread_local! {
    static WAS: Cell<Option<&'static AnyClass>> = const { Cell::new(None) };
}

pub fn keys_without_activating(ns_view: NonNull<c_void>) -> bool {
    if MainThreadMarker::new().is_none() {
        return false;
    }
    let view: &NSView = unsafe { ns_view.cast::<NSView>().as_ref() };
    let Some(window) = view.window() else {
        return false;
    };
    if !window.isKindOfClass(Floating::class()) && !floats(&window) {
        return false;
    }
    window.makeKeyAndOrderFront(None);
    window.isKeyWindow()
}

fn floats(window: &NSWindow) -> bool {
    let object: &AnyObject = window.as_ref();
    if !fits(Floating::class(), object.class()) {
        return false;
    }
    WAS.with(|was| was.set(Some(object.class())));
    unsafe { AnyObject::set_class(object, Floating::class()) };
    window.setStyleMask(window.styleMask() | NSWindowStyleMask::NonactivatingPanel);
    window.setCollectionBehavior(
        window.collectionBehavior()
            | NSWindowCollectionBehavior::MoveToActiveSpace
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    window.setHidesOnDeactivate(false);
    window.setAnimationBehavior(NSWindowAnimationBehavior::None);
    true
}

fn fits(panel: &AnyClass, window: &AnyClass) -> bool {
    panel.instance_size() <= window.instance_size()
}

pub fn grounded(ns_view: NonNull<c_void>) -> bool {
    if MainThreadMarker::new().is_none() {
        return false;
    }
    let view: &NSView = unsafe { ns_view.cast::<NSView>().as_ref() };
    let Some(window) = view.window() else {
        return false;
    };
    let Some(was) = WAS.with(Cell::get) else {
        return false;
    };
    if !window.isKindOfClass(Floating::class()) {
        return false;
    }
    let object: &AnyObject = window.as_ref();
    unsafe { AnyObject::set_class(object, was) };
    true
}

pub fn a_panel_would_float() -> Option<Floated> {
    let mtm = MainThreadMarker::new()?;
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(40.0, 40.0)),
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    unsafe { window.setReleasedWhenClosed(false) };
    let looker: Retained<Looker> = unsafe { msg_send![Looker::alloc(mtm), init] };
    let looked = NSString::from_str("effectiveAppearance");
    unsafe {
        window.addObserver_forKeyPath_options_context(
            &looker,
            &looked,
            NSKeyValueObservingOptions::New,
            std::ptr::null_mut(),
        );
    }
    let view = window.contentView()?;
    let at = NonNull::from(&*view).cast();
    let took_the_keys = keys_without_activating(at);
    let behaviour = window.collectionBehavior();
    let floated = Floated {
        became_a_panel: window.isKindOfClass(Floating::class()),
        never_activates: window
            .styleMask()
            .contains(NSWindowStyleMask::NonactivatingPanel),
        joins_full_screen: behaviour.contains(NSWindowCollectionBehavior::FullScreenAuxiliary),
        follows_the_space: behaviour.contains(NSWindowCollectionBehavior::MoveToActiveSpace),
        stays_when_left: !window.hidesOnDeactivate(),
        appears_at_once: window.animationBehavior() == NSWindowAnimationBehavior::None,
        took_the_keys,
        counts_as_ours: crate::activation::is_ours_up_front() == Some(true),
        back_as_it_was: grounded(at) && !window.isKindOfClass(Floating::class()),
    };
    window.orderOut(None);
    unsafe { window.removeObserver_forKeyPath(&looker, &looked) };
    window.close();
    Some(floated)
}

#[cfg(test)]
#[path = "floating_test.rs"]
mod tests;
