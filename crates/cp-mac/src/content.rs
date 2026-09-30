use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_core::paste_as::Content;
use std::borrow::Cow;

pub fn content_of<'a>(item: &'a Item, ocr: Option<&'a str>) -> Content<'a> {
    let bytes = |id: &str| {
        item.format(id).and_then(|format| match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.as_slice()),
            _ => None,
        })
    };
    let text = |id: &str| bytes(id).and_then(|bytes| std::str::from_utf8(bytes).ok());
    let html = text("public.html");
    Content {
        kind: item.kind,
        text: text("public.utf8-plain-text")
            .or_else(|| text(SYNTHETIC_TEXT))
            .map(Cow::Borrowed),
        html: html.map(Cow::Borrowed),
        rich: html.is_some()
            || bytes("public.rtf").is_some()
            || bytes("com.apple.flat-rtfd").is_some(),
        image: bytes("public.png")
            .or_else(|| bytes("public.tiff"))
            .or_else(|| bytes(SYNTHETIC_IMAGE)),
        paths: text("public.file-url")
            .map(|urls| {
                urls.lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(cp_mac_sys::files::path_of)
                    .collect()
            })
            .unwrap_or_default(),
        title: text("public.url-name").map(Cow::Borrowed),
        ocr,
    }
}

#[cfg(test)]
#[path = "content_test.rs"]
mod tests;
