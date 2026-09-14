use crate::formats::CATALOG;
use cp_core::formats::{Family, Take};
use cp_core::item::{Format, Item, Payload};
use cp_core::kind::{self, Kind};
use cp_mac_sys::pasteboard::Pasteboard;

pub fn capture(pb: &Pasteboard) -> Option<Item> {
    let offered = pb.types();
    let ids: Vec<&str> = offered.iter().map(String::as_str).collect();
    if CATALOG.refusal(&ids).is_some() {
        return None;
    }

    let family = CATALOG.classify(&ids);
    let mut formats: Vec<Format> = Vec::new();

    for id in &ids {
        let canonical = CATALOG.canonical(id);
        if formats.iter().any(|kept| kept.id == canonical) {
            continue;
        }
        let payload = match CATALOG.decide(canonical) {
            Take::Never => Payload::Announced { size: None },
            Take::Presence => Payload::Announced { size: None },
            Take::Payload => {
                if CATALOG.costlier_twin(canonical, &ids) {
                    Payload::Announced { size: None }
                } else if canonical == "public.file-url" {
                    match gather_file_urls(pb) {
                        Some(bytes) => Payload::stored(bytes),
                        None => Payload::Absent,
                    }
                } else {
                    match pb.data(canonical) {
                        Some(bytes) => Payload::stored(bytes),
                        None => Payload::Absent,
                    }
                }
            }
        };
        formats.push(Format {
            id: canonical.to_string(),
            payload,
        });
    }

    let kind = refine(family, &formats);
    Some(Item { kind, formats })
}

fn gather_file_urls(pb: &Pasteboard) -> Option<Vec<u8>> {
    let each = pb.data_per_item("public.file-url");
    if each.is_empty() {
        return pb.data("public.file-url");
    }
    let joined: Vec<String> = each
        .iter()
        .filter_map(|bytes| String::from_utf8(bytes.clone()).ok())
        .collect();
    (!joined.is_empty()).then(|| joined.join("\n").into_bytes())
}

fn refine(family: Option<Family>, formats: &[Format]) -> Option<Kind> {
    match family? {
        Family::Image => Some(Kind::Image),
        Family::Text => {
            let text = formats
                .iter()
                .find(|one| one.id == "public.utf8-plain-text")
                .and_then(|one| match &one.payload {
                    Payload::Inline(bytes) | Payload::Blob(bytes) => {
                        std::str::from_utf8(bytes).ok()
                    }
                    _ => None,
                });
            Some(text.map_or(Kind::Text, kind::classify_text))
        }
        Family::Files => {
            let first = formats
                .iter()
                .find(|one| one.id == "public.file-url")
                .and_then(|one| match &one.payload {
                    Payload::Inline(bytes) | Payload::Blob(bytes) => {
                        std::str::from_utf8(bytes).ok()
                    }
                    _ => None,
                });
            Some(match first.and_then(|urls| urls.lines().next()) {
                Some(url) => {
                    let path = url.trim_end_matches('/');
                    let name = path.rsplit('/').next().unwrap_or(path);
                    kind::classify_file(name, url.ends_with('/'))
                }
                None => Kind::File,
            })
        }
    }
}
