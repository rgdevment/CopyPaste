use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
use objc2_foundation::{MainThreadMarker, NSString, NSURL};

pub struct Pasteboard {
    inner: objc2::rc::Retained<NSPasteboard>,
}

impl Pasteboard {
    pub fn general(_mtm: MainThreadMarker) -> Self {
        Self {
            inner: NSPasteboard::generalPasteboard(),
        }
    }

    pub fn general_from_any_thread() -> Self {
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

pub fn file_path_of(url: &str) -> Option<String> {
    let parsed = NSURL::URLWithString(&NSString::from_str(url))?;
    if !parsed.isFileURL() {
        return None;
    }
    let path = parsed.filePathURL()?.absoluteString()?;
    Some(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_of(path: &std::path::Path) -> String {
        NSURL::fileURLWithPath(&NSString::from_str(&path.display().to_string()))
            .fileReferenceURL()
            .and_then(|url| url.absoluteString())
            .map(|url| url.to_string())
            .expect("un archivo que existe tiene referencia")
    }

    #[test]
    fn a_finder_reference_becomes_the_path_it_points_at() {
        let dir = std::env::temp_dir().join(format!("cp-ref-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("foto de perfil.png");
        std::fs::write(&file, b"png").unwrap();
        let reference = reference_of(&file);
        assert!(reference.starts_with("file:///.file/id="), "{reference}");
        let resolved = file_path_of(&reference).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert!(resolved.starts_with("file:///"), "{resolved}");
        assert!(resolved.ends_with("/foto%20de%20perfil.png"), "{resolved}");
        assert!(!resolved.contains(".file/id="));
    }

    #[test]
    fn a_path_url_comes_back_as_it_was() {
        assert_eq!(
            file_path_of("file:///tmp/uno.txt").as_deref(),
            Some("file:///tmp/uno.txt")
        );
        assert_eq!(
            file_path_of("file:///tmp/con%20espacio.txt").as_deref(),
            Some("file:///tmp/con%20espacio.txt")
        );
    }

    #[test]
    fn what_is_not_a_file_has_no_path() {
        assert_eq!(file_path_of("https://example.com/x.png"), None);
        assert_eq!(file_path_of("no es una url"), None);
        assert_eq!(file_path_of(""), None);
    }

    #[test]
    fn a_dead_reference_has_no_path_either() {
        let file = std::env::temp_dir().join(format!("cp-dead-{}.txt", std::process::id()));
        std::fs::write(&file, b"efimero").unwrap();
        let reference = reference_of(&file);
        std::fs::remove_file(&file).unwrap();
        assert_eq!(file_path_of(&reference), None);
    }
}
