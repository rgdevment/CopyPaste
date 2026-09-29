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
#[path = "identity_test.rs"]
mod tests;
