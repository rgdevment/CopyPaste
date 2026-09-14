use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
use objc2_foundation::{MainThreadMarker, NSString};

pub struct Pasteboard {
    inner: objc2::rc::Retained<NSPasteboard>,
}

impl Pasteboard {
    pub fn general(_mtm: MainThreadMarker) -> Self {
        Self {
            inner: NSPasteboard::generalPasteboard(),
        }
    }

    pub fn change_count(&self) -> i64 {
        self.inner.changeCount() as i64
    }

    pub fn types(&self) -> Vec<String> {
        let Some(types) = self.inner.types() else {
            return Vec::new();
        };
        types.iter().map(|one| one.to_string()).collect()
    }

    pub fn data(&self, uti: &str) -> Option<Vec<u8>> {
        let name = NSString::from_str(uti);
        let data = self.inner.dataForType(&name)?;
        Some(data.to_vec())
    }

    pub fn data_per_item(&self, uti: &str) -> Vec<Vec<u8>> {
        let Some(items) = self.inner.pasteboardItems() else {
            return Vec::new();
        };
        let name = NSString::from_str(uti);
        items
            .iter()
            .filter_map(|item| item.dataForType(&name).map(|data| data.to_vec()))
            .collect()
    }

    pub fn item_count(&self) -> usize {
        self.inner.pasteboardItems().map_or(0, |items| items.len())
    }

    pub fn write_items(&self, items: &[Vec<(&str, &str)>]) -> bool {
        self.inner.clearContents();
        let written: Vec<objc2::rc::Retained<NSPasteboardItem>> = items
            .iter()
            .map(|entries| {
                let item = NSPasteboardItem::new();
                for (uti, value) in entries {
                    let name = NSString::from_str(uti);
                    let text = NSString::from_str(value);
                    item.setString_forType(&text, &name);
                }
                item
            })
            .collect();
        let refs: Vec<&objc2::runtime::ProtocolObject<dyn NSPasteboardWriting>> = written
            .iter()
            .map(|item| objc2::runtime::ProtocolObject::from_ref(&**item))
            .collect();
        let array = objc2_foundation::NSArray::from_slice(&refs);
        self.inner.writeObjects(&array)
    }

    pub fn write_data(&self, uti: &str, bytes: &[u8]) -> bool {
        self.inner.clearContents();
        let name = NSString::from_str(uti);
        let data = objc2_foundation::NSData::with_bytes(bytes);
        self.inner.setData_forType(Some(&data), &name)
    }

    pub fn write_all(&self, entries: &[(&str, &[u8])]) -> bool {
        self.inner.clearContents();
        entries.iter().all(|(uti, bytes)| {
            let name = NSString::from_str(uti);
            let data = objc2_foundation::NSData::with_bytes(bytes);
            self.inner.setData_forType(Some(&data), &name)
        })
    }

    pub fn write_text(&self, text: &str) -> bool {
        self.inner.clearContents();
        let name = NSString::from_str("public.utf8-plain-text");
        let value = NSString::from_str(text);
        self.inner.setString_forType(&value, &name)
    }

    pub fn write_types(&self, entries: &[(&str, &str)]) -> bool {
        self.inner.clearContents();
        entries.iter().all(|(uti, value)| {
            let name = NSString::from_str(uti);
            let text = NSString::from_str(value);
            self.inner.setString_forType(&text, &name)
        })
    }
}

pub fn change_count_from_any_thread() -> i64 {
    objc2_app_kit::NSPasteboard::generalPasteboard().changeCount() as i64
}
