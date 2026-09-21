use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_core::paste_as::Content;

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
        text: text("public.utf8-plain-text").or_else(|| text(SYNTHETIC_TEXT)),
        html,
        rich: html.is_some()
            || bytes("public.rtf").is_some()
            || bytes("com.apple.flat-rtfd").is_some(),
        png: bytes("public.png").or_else(|| bytes(SYNTHETIC_IMAGE)),
        paths: text("public.file-url")
            .map(|urls| {
                urls.lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(cp_mac_sys::frontmost::path_of)
                    .collect()
            })
            .unwrap_or_default(),
        title: text("public.url-name"),
        ocr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cp_core::item::Format;
    use cp_core::kind::Kind;
    use cp_core::paste_as::{Form, forms_for};

    fn inline(id: &str, bytes: &[u8]) -> Format {
        Format {
            id: id.into(),
            payload: Payload::Inline(bytes.to_vec()),
        }
    }

    #[test]
    fn a_word_paragraph_is_rich_with_html_and_plain_text() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("public.rtf", b"{\\rtf1 hola}"),
                inline("public.html", b"<p><b>hola</b></p>"),
                inline("public.utf8-plain-text", b"hola"),
                Format {
                    id: "public.tiff".into(),
                    payload: Payload::Announced { size: None },
                },
            ],
        };
        let content = content_of(&item, None);
        assert_eq!(content.text, Some("hola"));
        assert_eq!(content.html, Some("<p><b>hola</b></p>"));
        assert!(content.rich);
        assert_eq!(content.png, None);
        assert_eq!(
            forms_for(&content),
            vec![Form::AsIs, Form::PlainText, Form::Markdown]
        );
    }

    #[test]
    fn html_alone_is_enough_to_be_rich() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("public.html", b"<i>hola</i>"),
                inline("public.utf8-plain-text", b"hola"),
            ],
        };
        let content = content_of(&item, None);
        assert!(content.rich);
        assert_eq!(content.html, Some("<i>hola</i>"));
    }

    #[test]
    fn an_rtf_only_copy_is_rich_but_has_no_markdown() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("com.apple.flat-rtfd", b"rtfd"),
                inline("public.utf8-plain-text", b"hola"),
            ],
        };
        let content = content_of(&item, None);
        assert!(content.rich);
        assert_eq!(content.html, None);
        assert_eq!(forms_for(&content), vec![Form::AsIs, Form::PlainText]);
    }

    #[test]
    fn finder_urls_become_paths_and_a_link_brings_its_title() {
        let files = Item {
            kind: Some(Kind::File),
            formats: vec![inline(
                "public.file-url",
                b"file:///tmp/cp%20a9.png\nfile:///tmp/dos.txt\n",
            )],
        };
        let content = content_of(&files, None);
        assert_eq!(content.paths, vec!["/tmp/cp a9.png", "/tmp/dos.txt"]);
        assert_eq!(content.text, None);

        let link = Item {
            kind: Some(Kind::Link),
            formats: vec![
                inline("public.utf8-plain-text", b"https://ejemplo.test"),
                inline("public.url", b"https://ejemplo.test"),
                inline("public.url-name", "Ejemplo — inicio".as_bytes()),
            ],
        };
        let content = content_of(&link, None);
        assert_eq!(content.title, Some("Ejemplo — inicio"));
        assert!(forms_for(&content).contains(&Form::LinkTitled));
    }

    #[test]
    fn a_stored_image_and_what_was_read_in_it() {
        let item = Item {
            kind: Some(Kind::Image),
            formats: vec![inline("public.png", &[137, 80, 78, 71])],
        };
        let content = content_of(&item, Some("Pedido 4417"));
        assert_eq!(content.png, Some(&[137u8, 80, 78, 71][..]));
        assert_eq!(content.ocr, Some("Pedido 4417"));
        assert!(!content.rich);
        assert_eq!(
            forms_for(&content),
            vec![Form::AsIs, Form::ImageJpeg, Form::ImageOcr]
        );
    }

    #[test]
    fn what_the_store_wrote_itself_is_read_back_the_same_way() {
        let edited = Item::plain("editado a mano");
        assert_eq!(content_of(&edited, None).text, Some("editado a mano"));
        let synthetic = Item {
            kind: Some(Kind::Image),
            formats: vec![inline(SYNTHETIC_IMAGE, &[1, 2, 3])],
        };
        assert_eq!(content_of(&synthetic, None).png, Some(&[1u8, 2, 3][..]));
    }

    #[test]
    fn bytes_that_are_not_text_do_not_pretend_to_be() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![inline("public.utf8-plain-text", &[0xff, 0xfe, 0x00])],
        };
        assert_eq!(content_of(&item, None).text, None);
    }
}
