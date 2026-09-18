use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{CFArray, CFBoolean, CFNumber, CFRetained, CFString, CFType};
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuFailure {
    NoMenuBar,
    NoPasteItem,
    PasteDisabled,
    PressRefused(i32),
}

pub fn press_paste(pid: i32) -> Result<(), MenuFailure> {
    let app = unsafe { AXUIElement::new_application(pid) };
    let bar = element(&app, "AXMenuBar").ok_or(MenuFailure::NoMenuBar)?;
    let item = children(&bar)
        .into_iter()
        .flat_map(|menu| children(&menu))
        .flat_map(|submenu| children(&submenu))
        .find(|entry| is_paste_item(entry))
        .ok_or(MenuFailure::NoPasteItem)?;
    if !enabled(&item) {
        return Err(MenuFailure::PasteDisabled);
    }
    let pressed = unsafe { item.perform_action(&CFString::from_str("AXPress")) };
    if pressed == AXError::Success {
        Ok(())
    } else {
        Err(MenuFailure::PressRefused(pressed.0))
    }
}

pub fn is_paste_shortcut(command_char: &str, modifiers: u32) -> bool {
    command_char.eq_ignore_ascii_case("v") && modifiers == 0
}

fn is_paste_item(entry: &AXUIElement) -> bool {
    let Some(character) = attribute(entry, "AXMenuItemCmdChar") else {
        return false;
    };
    let Some(character) = character.downcast_ref::<CFString>() else {
        return false;
    };
    let modifiers = attribute(entry, "AXMenuItemCmdModifiers")
        .and_then(|value| value.downcast_ref::<CFNumber>()?.as_i32())
        .unwrap_or(0);
    is_paste_shortcut(&character.to_string(), modifiers.unsigned_abs())
}

fn enabled(entry: &AXUIElement) -> bool {
    attribute(entry, "AXEnabled")
        .and_then(|value| Some(value.downcast_ref::<CFBoolean>()?.as_bool()))
        .unwrap_or(false)
}

fn element(parent: &AXUIElement, name: &str) -> Option<CFRetained<AXUIElement>> {
    attribute(parent, name)?.downcast::<AXUIElement>().ok()
}

fn children(parent: &AXUIElement) -> Vec<CFRetained<AXUIElement>> {
    let Some(array) =
        attribute(parent, "AXChildren").and_then(|value| value.downcast::<CFArray>().ok())
    else {
        return Vec::new();
    };
    let objects = unsafe { array.cast_unchecked::<CFType>() };
    objects
        .to_vec()
        .into_iter()
        .filter_map(|child| child.downcast::<AXUIElement>().ok())
        .collect()
}

fn attribute(element: &AXUIElement, name: &str) -> Option<CFRetained<CFType>> {
    let mut value: *const CFType = std::ptr::null();
    let asked = unsafe {
        element.copy_attribute_value(&CFString::from_str(name), NonNull::from(&mut value))
    };
    if asked != AXError::Success {
        return None;
    }
    let raw = NonNull::new(value.cast_mut())?;
    Some(unsafe { CFRetained::from_raw(raw) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_is_command_v_and_nothing_else() {
        assert!(is_paste_shortcut("v", 0));
        assert!(is_paste_shortcut("V", 0));
        assert!(
            !is_paste_shortcut("v", 1),
            "⇧⌘V es «pegar con el mismo estilo»"
        );
        assert!(!is_paste_shortcut("v", 2), "⌥⌘V es otra cosa");
        assert!(!is_paste_shortcut("c", 0));
        assert!(!is_paste_shortcut("", 0));
    }

    #[test]
    fn a_process_that_does_not_exist_has_no_menu_bar() {
        assert_eq!(press_paste(i32::MAX), Err(MenuFailure::NoMenuBar));
    }
}
