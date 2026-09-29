use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_core::paste_as::Content;
use cp_win_sys::writing::text_of;
use std::borrow::Cow;

use crate::drop::paths_in;

pub fn content_of<'a>(item: &'a Item, ocr: Option<&'a str>) -> Content<'a> {
    let bytes = |id: &str| {
        item.format(id).and_then(|format| match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.as_slice()),
            _ => None,
        })
    };
    let utf8 = |id: &str| bytes(id).and_then(|bytes| std::str::from_utf8(bytes).ok());
    let html = utf8("HTML Format");
    Content {
        kind: item.kind,
        text: bytes("CF_UNICODETEXT")
            .and_then(text_of)
            .map(Cow::Owned)
            .or_else(|| utf8(SYNTHETIC_TEXT).map(Cow::Borrowed)),
        html: html.map(Cow::Borrowed),
        rich: html.is_some() || bytes("Rich Text Format").is_some(),
        image: bytes("PNG").or_else(|| bytes(SYNTHETIC_IMAGE)),
        paths: bytes("CF_HDROP").map(paths_in).unwrap_or_default(),
        title: None,
        ocr,
    }
}

#[cfg(test)]
#[path = "content_test.rs"]
mod tests;
