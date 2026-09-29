use super::*;

const DOCS: &str = "<meta charset=\"utf-8\"><b style=\"font-weight:normal;\" \
        id=\"docs-internal-guid-4a1e6b2f-7fff-1d3e-8c5a-2b3c4d5e6f70\"><span>hi</span></b>";

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
        "on Windows the same HTML arrives under another name"
    );
    let kept = stable("public.html", DOCS.as_bytes());
    assert!(
        std::str::from_utf8(&kept)
            .unwrap()
            .contains("id=\"docs-internal-guid-\"><span>hi</span>"),
        "the marker is left without its variable part, and the rest untouched"
    );
}

#[test]
fn markup_without_anything_volatile_is_borrowed_not_copied() {
    let html = b"<b>hi</b>";
    assert!(matches!(stable("public.html", html), Cow::Borrowed(_)));
    assert_eq!(stable("public.html", html).as_ref(), html);
}

#[test]
fn only_markup_is_looked_at() {
    let text = b"paste docs-internal-guid-1234 as is";
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
    let html = b"<b id=\"docs-internal-guid-ab-12\">x</b><i id=\"docs-internal-guid-cd\">y</i>z";
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
        "the GUID may come in uppercase"
    );
}
