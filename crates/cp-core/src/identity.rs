use std::borrow::Cow;

const VOLATILE: &[&[u8]] = &[b"docs-internal-guid-"];

const RENDERINGS: &[&str] = &[
    "public.rtf",
    "com.apple.flat-rtfd",
    "com.apple.webarchive",
    "com.adobe.pdf",
    "text/rtf",
    "application/pdf",
    "rich text format",
];

pub fn bears_identity(id: &str) -> bool {
    !RENDERINGS.contains(&id.to_ascii_lowercase().as_str())
}

pub fn is_markup(id: &str) -> bool {
    id.to_ascii_lowercase().contains("html")
}

pub fn stable<'a>(id: &str, bytes: &'a [u8]) -> Cow<'a, [u8]> {
    if !is_markup(id) {
        return Cow::Borrowed(bytes);
    }
    let Some((first, marker)) = next_volatile(bytes) else {
        return Cow::Borrowed(bytes);
    };
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    let mut found = Some((first, marker));
    while let Some((start, marker)) = found {
        let end = at + start + marker.len();
        out.extend_from_slice(&bytes[at..end]);
        at = end;
        while at < bytes.len() && (bytes[at].is_ascii_hexdigit() || bytes[at] == b'-') {
            at += 1;
        }
        found = next_volatile(&bytes[at..]);
    }
    out.extend_from_slice(&bytes[at..]);
    Cow::Owned(out)
}

fn next_volatile(bytes: &[u8]) -> Option<(usize, &'static [u8])> {
    VOLATILE
        .iter()
        .filter_map(|marker| find(bytes, marker).map(|at| (at, *marker)))
        .min_by_key(|(at, _)| *at)
}

fn find(bytes: &[u8], marker: &[u8]) -> Option<usize> {
    bytes
        .windows(marker.len())
        .position(|window| window == marker)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCS: &str = "<meta charset=\"utf-8\"><b style=\"font-weight:normal;\" \
        id=\"docs-internal-guid-4a1e6b2f-7fff-1d3e-8c5a-2b3c4d5e6f70\"><span>hola</span></b>";

    #[test]
    fn the_guid_google_writes_on_every_copy_leaves_the_identity() {
        let again = DOCS.replace(
            "4a1e6b2f-7fff-1d3e-8c5a-2b3c4d5e6f70",
            "0c9d8e7f-7fff-aaaa-bbbb-000000000001",
        );
        assert_ne!(DOCS, again);
        assert_eq!(
            stable("public.html", DOCS.as_bytes()),
            stable("public.html", again.as_bytes())
        );
        assert_eq!(
            stable("HTML Format", DOCS.as_bytes()),
            stable("HTML Format", again.as_bytes()),
            "en Windows el mismo HTML llega bajo otro nombre"
        );
        let kept = stable("public.html", DOCS.as_bytes());
        assert!(
            std::str::from_utf8(&kept)
                .unwrap()
                .contains("id=\"docs-internal-guid-\"><span>hola</span>"),
            "queda el marcador sin su parte variable, y el resto intacto"
        );
    }

    #[test]
    fn markup_without_anything_volatile_is_borrowed_not_copied() {
        let html = b"<b>hola</b>";
        assert!(matches!(stable("public.html", html), Cow::Borrowed(_)));
        assert_eq!(stable("public.html", html).as_ref(), html);
    }

    #[test]
    fn only_markup_is_looked_at() {
        let text = b"pega docs-internal-guid-1234 tal cual";
        assert!(matches!(
            stable("public.utf8-plain-text", text),
            Cow::Borrowed(_)
        ));
        assert!(matches!(stable("public.png", text), Cow::Borrowed(_)));
        assert!(matches!(stable("public.html", text), Cow::Owned(_)));
        assert!(matches!(stable("text/html", text), Cow::Owned(_)));
        assert!(is_markup("HTML Format"));
        assert!(!is_markup("public.rtf"));
        assert!(!is_markup("com.apple.webarchive"));
    }

    #[test]
    fn a_rendering_of_the_content_does_not_carry_its_identity() {
        for id in [
            "public.rtf",
            "com.apple.flat-rtfd",
            "com.apple.webarchive",
            "com.adobe.pdf",
            "Rich Text Format",
            "text/rtf",
        ] {
            assert!(!bears_identity(id), "{id}");
        }
        for id in [
            "public.utf8-plain-text",
            "public.html",
            "HTML Format",
            "public.png",
            "public.tiff",
            "public.file-url",
            "text/plain",
            "image/png",
        ] {
            assert!(bears_identity(id), "{id}");
        }
    }

    #[test]
    fn every_occurrence_goes_and_what_follows_the_guid_stays() {
        let html =
            b"<b id=\"docs-internal-guid-ab-12\">x</b><i id=\"docs-internal-guid-cd\">y</i>z";
        let kept = stable("public.html", html);
        assert_eq!(
            kept.as_ref(),
            b"<b id=\"docs-internal-guid-\">x</b><i id=\"docs-internal-guid-\">y</i>z"
        );
        let at_the_end = b"<b id=\"docs-internal-guid-ffff";
        assert_eq!(
            stable("public.html", at_the_end).as_ref(),
            b"<b id=\"docs-internal-guid-"
        );
        let upper = b"id=\"docs-internal-guid-ABCDEF\">";
        assert_eq!(
            stable("public.html", upper).as_ref(),
            b"id=\"docs-internal-guid-\">",
            "el GUID puede venir en mayúsculas"
        );
    }
}
