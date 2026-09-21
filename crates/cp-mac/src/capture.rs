use crate::formats::CATALOG;
use cp_core::formats::{Family, Take};
use cp_core::item::{Format, Item, Payload};
use cp_core::kind::{self, Kind};
use cp_mac_sys::pasteboard::{self, Pasteboard};
use cp_mac_sys::reading;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Kept(Item),
    Nothing,
    TooSlow,
    Superseded,
}

pub const PATIENCE: std::time::Duration = std::time::Duration::from_millis(400);

const _: () = assert!(PATIENCE.as_millis() < 60_000);

pub fn capture_within(patience: std::time::Duration) -> Captured {
    reading::anything_within(patience, || {
        capture(&Pasteboard::general_from_any_thread()).map_or(Captured::Nothing, Captured::Kept)
    })
    .unwrap_or(Captured::TooSlow)
}

pub fn capture_insisting(patience: std::time::Duration, retry: cp_core::watch::Retry) -> Captured {
    insisting(retry, pasteboard::change_count_from_any_thread, || {
        capture_within(patience)
    })
}

fn insisting(
    retry: cp_core::watch::Retry,
    count: impl Fn() -> i64,
    mut once: impl FnMut() -> Captured,
) -> Captured {
    use cp_core::watch::{Retried, insist};
    let attempt = || match once() {
        Captured::TooSlow => None,
        other => Some(other),
    };
    match insist(retry, count, attempt, std::thread::sleep) {
        Retried::Done(captured) => captured,
        Retried::Superseded => Captured::Superseded,
        Retried::Exhausted => Captured::TooSlow,
    }
}

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
        let payload = match CATALOG.decide_in(family, canonical) {
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
                        Some(bytes) if bytes.is_empty() && is_image(canonical) => Payload::Absent,
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
    let mut each = pb.data_per_item("public.file-url");
    if each.is_empty() {
        each.push(pb.data("public.file-url")?);
    }
    let resolved: Vec<String> = each
        .into_iter()
        .filter_map(|bytes| String::from_utf8(bytes).ok())
        .map(|url| pasteboard::file_path_of(&url).unwrap_or(url))
        .collect();
    (!resolved.is_empty()).then(|| resolved.join("\n").into_bytes())
}

fn is_image(id: &str) -> bool {
    CATALOG.images_by_preference.contains(&id)
}

fn has_bytes(one: &Format) -> bool {
    one.payload.size().is_some_and(|bytes| bytes > 0)
}

fn refine(family: Option<Family>, formats: &[Format]) -> Option<Kind> {
    let image_delivered = formats
        .iter()
        .any(|one| is_image(&one.id) && has_bytes(one));
    let text_delivered = formats
        .iter()
        .any(|one| one.id == "public.utf8-plain-text" && has_bytes(one));
    let family = match family? {
        Family::Image if !image_delivered && text_delivered => Family::Text,
        other => other,
    };
    match family {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn quick() -> cp_core::watch::Retry {
        cp_core::watch::Retry {
            attempts: 3,
            pause: std::time::Duration::from_millis(1),
        }
    }

    #[test]
    fn a_slow_source_that_answers_on_the_second_try_is_kept() {
        let mut tries = 0;
        let got = insisting(
            quick(),
            || 7,
            || {
                tries += 1;
                if tries < 2 {
                    Captured::TooSlow
                } else {
                    Captured::Kept(Item::plain("tarde pero llega"))
                }
            },
        );
        assert_eq!(got, Captured::Kept(Item::plain("tarde pero llega")));
    }

    #[test]
    fn a_source_that_stays_slow_is_too_slow_in_the_end() {
        let mut tries = 0;
        let got = insisting(
            quick(),
            || 7,
            || {
                tries += 1;
                Captured::TooSlow
            },
        );
        assert_eq!(got, Captured::TooSlow);
        assert_eq!(tries, 3);
    }

    #[test]
    fn an_empty_pasteboard_is_an_answer_not_a_delay() {
        let mut tries = 0;
        let got = insisting(
            quick(),
            || 7,
            || {
                tries += 1;
                Captured::Nothing
            },
        );
        assert_eq!(got, Captured::Nothing);
        assert_eq!(tries, 1);
    }

    #[test]
    fn a_copy_made_while_insisting_supersedes_the_slow_one() {
        let count = std::cell::Cell::new(7);
        let got = insisting(
            quick(),
            || count.get(),
            || {
                count.set(8);
                Captured::TooSlow
            },
        );
        assert_eq!(got, Captured::Superseded);
    }

    #[test]
    fn the_patience_sits_between_the_two_measured_worlds() {
        let delivered = std::time::Duration::from_millis(9);
        let promised = std::time::Duration::from_secs(60);
        assert!(delivered < PATIENCE, "Finder entrega 16 tipos en 9 ms");
        assert!(PATIENCE < std::time::Duration::from_secs(1));
        assert!(PATIENCE < promised, "una promesa tarda 60 s en rendirse");
    }

    #[test]
    fn a_capture_that_does_not_finish_in_time_is_abandoned() {
        let seen = reading::anything_within(std::time::Duration::from_millis(20), || {
            std::thread::sleep(std::time::Duration::from_secs(60));
            Captured::Nothing
        });
        assert_eq!(seen, None, "el hilo se abandona y no se espera");
    }
}
