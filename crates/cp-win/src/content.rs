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
mod tests {
    use super::*;
    use crate::drop::drop_of;
    use cp_core::item::Format;
    use cp_core::kind::Kind;
    use cp_core::paste_as::{Form, Rendered, forms_for, render};
    use cp_win_sys::writing::utf16_of;

    const WORD_HTML: &str = "Version:0.9\r\nStartHTML:0000000105\r\nEndHTML:0000000200\r\nStartFragment:0000000141\r\nEndFragment:0000000164\r\n<html><body><!--StartFragment--><p><b>hola</b></p><!--EndFragment--></body></html>";

    fn inline(id: &str, bytes: &[u8]) -> Format {
        Format {
            id: id.into(),
            payload: Payload::Inline(bytes.to_vec()),
        }
    }

    fn unicode(text: &str) -> Format {
        inline("CF_UNICODETEXT", &utf16_of(text))
    }

    #[test]
    fn a_word_paragraph_is_rich_with_html_and_plain_text() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("Rich Text Format", b"{\\rtf1 hola}"),
                inline("HTML Format", WORD_HTML.as_bytes()),
                unicode("hola"),
                Format {
                    id: "CF_DIBV5".into(),
                    payload: Payload::Announced { size: None },
                },
            ],
        };
        let content = content_of(&item, None);
        assert_eq!(content.text.as_deref(), Some("hola"));
        assert_eq!(content.html.as_deref(), Some(WORD_HTML));
        assert!(content.rich);
        assert_eq!(content.image, None);
        assert_eq!(forms_for(&content), vec![Form::PlainText, Form::Markdown]);
    }

    #[test]
    fn the_clipboard_header_of_the_html_never_reaches_the_markdown() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![inline("HTML Format", WORD_HTML.as_bytes()), unicode("hola")],
        };
        let markdown = render(Form::Markdown, &content_of(&item, None));
        assert_eq!(markdown, Some(Rendered::Text("**hola**".into())));
    }

    #[test]
    fn html_alone_is_enough_to_be_rich() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![inline("HTML Format", b"<i>hola</i>"), unicode("hola")],
        };
        let content = content_of(&item, None);
        assert!(content.rich);
        assert_eq!(content.html.as_deref(), Some("<i>hola</i>"));
    }

    #[test]
    fn plain_rtf_alone_is_rich_and_a_blob_reads_like_an_inline_payload() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("Rich Text Format", b"{\\rtf1 hola}"),
                Format {
                    id: "CF_UNICODETEXT".into(),
                    payload: Payload::Blob(utf16_of("hola")),
                },
            ],
        };
        let content = content_of(&item, None);
        assert!(content.rich, "el RTF a solas ya es enriquecido");
        assert_eq!(
            content.text.as_deref(),
            Some("hola"),
            "un blob se lee igual que un inline"
        );
    }

    #[test]
    fn a_rich_text_with_an_image_offers_both_families() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("Rich Text Format", b"{\\rtf1 hola}"),
                unicode("hola"),
                inline("PNG", &[137, 80, 78, 71]),
            ],
        };
        assert_eq!(
            forms_for(&content_of(&item, None)),
            vec![Form::PlainText, Form::ImageJpeg],
            "un documento con una imagen incrustada se puede pegar como texto o como su imagen"
        );
    }

    #[test]
    fn an_rtf_only_copy_is_rich_but_has_no_markdown() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("Rich Text Format", b"{\\rtf1 hola}"),
                unicode("hola"),
            ],
        };
        let content = content_of(&item, None);
        assert!(content.rich);
        assert_eq!(content.html, None);
        assert_eq!(forms_for(&content), vec![Form::PlainText]);
    }

    #[test]
    fn a_drop_becomes_its_paths_and_a_link_carries_no_title() {
        let files = Item {
            kind: Some(Kind::File),
            formats: vec![inline(
                "CF_HDROP",
                &drop_of(&[r"C:\Mis Documentos\cp a9.png", r"C:\dos.txt"]),
            )],
        };
        let content = content_of(&files, None);
        assert_eq!(
            content.paths,
            vec![r"C:\Mis Documentos\cp a9.png", r"C:\dos.txt"]
        );
        assert_eq!(content.text, None);
        assert_eq!(forms_for(&content), vec![Form::Path]);

        let link = Item {
            kind: Some(Kind::Link),
            formats: vec![
                unicode("https://ejemplo.test"),
                inline("Chromium internal source URL", b"https://ejemplo.test"),
            ],
        };
        let content = content_of(&link, None);
        assert_eq!(content.title, None);
        let forms = forms_for(&content);
        assert!(forms.contains(&Form::LinkMarkdown));
        assert!(
            !forms.contains(&Form::LinkTitled),
            "sin título no hay forma titulada"
        );
    }

    #[test]
    fn a_stored_image_and_what_was_read_in_it() {
        let item = Item {
            kind: Some(Kind::Image),
            formats: vec![inline("PNG", &[137, 80, 78, 71])],
        };
        let content = content_of(&item, Some("Pedido 4417"));
        assert_eq!(content.image, Some(&[137u8, 80, 78, 71][..]));
        assert_eq!(content.ocr, Some("Pedido 4417"));
        assert!(!content.rich);
        assert_eq!(forms_for(&content), vec![Form::ImageJpeg, Form::ImageOcr]);
    }

    #[test]
    fn a_bitmap_only_copy_is_the_png_we_transcoded_from_it() {
        let item = Item {
            kind: Some(Kind::Image),
            formats: vec![
                Format {
                    id: "CF_DIBV5".into(),
                    payload: Payload::Announced { size: Some(4) },
                },
                inline("CF_DIB", &[40, 0, 0, 0]),
                inline(SYNTHETIC_IMAGE, &[137, 80, 78, 71]),
            ],
        };
        let content = content_of(&item, None);
        assert_eq!(
            content.image,
            Some(&[137u8, 80, 78, 71][..]),
            "el DIB crudo no se ofrece: la imagen es el PNG sintético"
        );
        let both = Item {
            kind: Some(Kind::Image),
            formats: vec![inline(SYNTHETIC_IMAGE, &[1]), inline("PNG", &[2])],
        };
        assert_eq!(
            content_of(&both, None).image,
            Some(&[2u8][..]),
            "el PNG de la fuente manda si lo hay"
        );
    }

    #[test]
    fn what_the_store_wrote_itself_is_read_back_the_same_way() {
        let edited = Item::plain("editado a mano");
        assert_eq!(
            content_of(&edited, None).text.as_deref(),
            Some("editado a mano")
        );
        let synthetic = Item {
            kind: Some(Kind::Image),
            formats: vec![inline(SYNTHETIC_IMAGE, &[1, 2, 3])],
        };
        assert_eq!(content_of(&synthetic, None).image, Some(&[1u8, 2, 3][..]));
    }

    #[test]
    fn the_system_text_wins_over_the_synthetic_one() {
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![inline(SYNTHETIC_TEXT, b"sintetico"), unicode("del sistema")],
        };
        assert_eq!(content_of(&item, None).text.as_deref(), Some("del sistema"));
    }

    #[test]
    fn bytes_that_are_not_text_do_not_pretend_to_be() {
        let lone_surrogate = [0x00, 0xD8, 0x00, 0x00];
        let item = Item {
            kind: Some(Kind::Text),
            formats: vec![
                inline("CF_UNICODETEXT", &lone_surrogate),
                inline("HTML Format", &[0xff, 0xfe, 0x00]),
            ],
        };
        let content = content_of(&item, None);
        assert_eq!(content.text, None);
        assert_eq!(content.html, None);
        assert!(!content.rich, "un HTML ilegible no hace rico al ítem");
    }
}
