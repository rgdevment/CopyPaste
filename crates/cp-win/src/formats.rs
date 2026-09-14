use cp_core::formats::Catalog;

pub const CATALOG: Catalog = Catalog {
    hangs: &[],
    wasteful: &[
        "Embed Source",
        "Native",
        "OwnerLink",
        "ObjectLink",
        "Link Source",
        "Link Source Descriptor",
        "Link",
        "Object Descriptor",
        "Ole Private Data",
        "DataObject",
        "DataObjectAttributes",
        "DataObjectAttributesRequiringElevation",
        "Shell Object Offsets",
        "AsyncFlag",
        "ZoneIdentifier",
        "CF_ENHMETAFILE",
        "CF_METAFILEPICT",
        "CF_SYLK",
        "CF_DIF",
        "Biff12",
        "Biff8",
        "Biff5",
        "XML Spreadsheet",
        "CF_BITMAP",
        "FileGroupDescriptorW",
        "FileContents",
        "Shell IDList Array",
        "Chromium internal source RFH token",
        "FileName",
        "FileNameW",
    ],
    wanted: &[
        "CF_UNICODETEXT",
        "Rich Text Format",
        "HTML Format",
        "PNG",
        "CF_DIBV5",
        "CF_DIB",
        "CF_HDROP",
        "Csv",
        "Preferred DropEffect",
        "Chromium internal source URL",
        "CanIncludeInClipboardHistory",
    ],
    aliases: &[],
    concealed: &[
        "Clipboard Viewer Ignore",
        "ExcludeClipboardContentFromMonitorProcessing",
        "ExcludeClipboardDataFromMonitorProcessing",
    ],
    denied_when_zero: &["CanIncludeInClipboardHistory"],
    opaque_prefixes: &[],
    text: &["CF_UNICODETEXT", "Rich Text Format", "HTML Format", "Csv"],
    files: &["CF_HDROP"],
    images_by_preference: &["PNG", "CF_DIBV5", "CF_DIB"],
    equivalents: &[
        &["CF_UNICODETEXT", "CF_TEXT", "CF_OEMTEXT"],
        &["HTML Format", "text/html"],
    ],
    embeddable: &[
        "Embed Source",
        "Native",
        "Object Descriptor",
        "Link Source Descriptor",
        "Biff12",
        "Biff8",
        "Biff5",
        "XML Spreadsheet",
        "Csv",
    ],
};

#[cfg(test)]
mod tests {
    use super::CATALOG;
    use cp_core::formats::{Family, Refusal, Take};

    const WORD: &[&str] = &[
        "DataObject",
        "Object Descriptor",
        "Rich Text Format",
        "HTML Format",
        "CF_TEXT",
        "CF_UNICODETEXT",
        "CF_ENHMETAFILE",
        "CF_METAFILEPICT",
        "Embed Source",
        "Native",
        "OwnerLink",
        "Link Source",
        "Link Source Descriptor",
        "ObjectLink",
        "Ole Private Data",
        "CF_LOCALE",
        "CF_OEMTEXT",
    ];

    const EXCEL: &[&str] = &[
        "DataObject",
        "CF_ENHMETAFILE",
        "CF_METAFILEPICT",
        "CF_BITMAP",
        "Biff12",
        "Biff8",
        "Biff5",
        "CF_SYLK",
        "CF_DIF",
        "XML Spreadsheet",
        "HTML Format",
        "CF_UNICODETEXT",
        "CF_TEXT",
        "Csv",
        "Rich Text Format",
        "Embed Source",
        "Native",
        "OwnerLink",
        "Object Descriptor",
        "Link Source",
        "Link Source Descriptor",
        "Link",
        "ExcludeClipboardContentFromMonitorProcessing",
        "ObjectLink",
        "Ole Private Data",
        "CF_LOCALE",
        "CF_OEMTEXT",
        "CF_DIB",
        "CF_DIBV5",
    ];

    const FIREFOX: &[&str] = &[
        "DataObject",
        "text/html",
        "HTML Format",
        "text/_moz_htmlcontext",
        "text/_moz_htmlinfo",
        "CF_UNICODETEXT",
        "CF_TEXT",
        "text/x-moz-url-priv",
        "Ole Private Data",
        "CF_LOCALE",
        "CF_OEMTEXT",
    ];

    const CHROME: &[&str] = &[
        "HTML Format",
        "CF_UNICODETEXT",
        "Chromium internal source RFH token",
        "Chromium internal source URL",
        "CF_LOCALE",
        "CF_TEXT",
        "CF_OEMTEXT",
    ];

    const EXPLORER: &[&str] = &[
        "DataObject",
        "Shell IDList Array",
        "DataObjectAttributes",
        "DataObjectAttributesRequiringElevation",
        "Shell Object Offsets",
        "Preferred DropEffect",
        "AsyncFlag",
        "CF_HDROP",
        "FileName",
        "FileContents",
        "FileNameW",
        "FileGroupDescriptorW",
        "ZoneIdentifier",
        "Ole Private Data",
    ];

    const SNIP: &[&str] = &[
        "DataObject",
        "CF_BITMAP",
        "PNG",
        "CanUploadToCloudClipboard",
        "CanIncludeInClipboardHistory",
        "Ole Private Data",
        "CF_DIB",
        "CF_DIBV5",
    ];

    const TERMINAL: &[&str] = &["CF_UNICODETEXT", "CF_LOCALE", "CF_TEXT", "CF_OEMTEXT"];

    fn kept(offered: &[&str]) -> Vec<&'static str> {
        offered
            .iter()
            .filter(|id| CATALOG.decide(id) == Take::Payload)
            .filter(|id| !CATALOG.costlier_twin(id, offered))
            .map(|id| {
                CATALOG
                    .wanted
                    .iter()
                    .find(|one| **one == CATALOG.canonical(id))
                    .copied()
                    .unwrap_or("?")
            })
            .collect()
    }

    #[test]
    fn word_keeps_its_styles_and_leaves_the_ole_object_alone() {
        assert_eq!(CATALOG.classify(WORD), Some(Family::Text));
        let kept = kept(WORD);
        assert!(kept.contains(&"Rich Text Format"), "{kept:?}");
        assert!(kept.contains(&"HTML Format"), "{kept:?}");
        assert!(kept.contains(&"CF_UNICODETEXT"), "{kept:?}");
        for heavy in ["Embed Source", "Native", "CF_ENHMETAFILE"] {
            assert_eq!(CATALOG.decide(heavy), Take::Presence, "{heavy}");
        }
    }

    #[test]
    fn a_spreadsheet_is_text_and_not_a_picture_of_itself() {
        assert_eq!(CATALOG.classify(EXCEL), Some(Family::Text));
    }

    #[test]
    fn excel_asks_not_to_be_recorded_and_says_so_by_name() {
        assert_eq!(
            CATALOG.refusal(EXCEL),
            Some(Refusal::Marked(
                "ExcludeClipboardContentFromMonitorProcessing"
            ))
        );
        assert_eq!(CATALOG.refusal(WORD), None, "Word no lo pide");
    }

    #[test]
    fn a_password_manager_that_says_zero_is_obeyed() {
        assert_eq!(
            CATALOG.declines("CanIncludeInClipboardHistory", &[0, 0, 0, 0]),
            Some(Refusal::Declined("CanIncludeInClipboardHistory"))
        );
        assert_eq!(
            CATALOG.declines("CanIncludeInClipboardHistory", &[1, 0, 0, 0]),
            None,
            "la captura de pantalla dice que sí y se guarda"
        );
    }

    #[test]
    fn a_browser_selection_is_not_stored_twice() {
        assert_eq!(CATALOG.classify(FIREFOX), Some(Family::Text));
        assert!(CATALOG.costlier_twin("text/html", FIREFOX));
        assert!(!CATALOG.costlier_twin("HTML Format", FIREFOX));
    }

    #[test]
    fn the_ansi_halves_of_the_text_are_never_kept() {
        for source in [WORD, EXCEL, FIREFOX, CHROME, TERMINAL] {
            for degraded in ["CF_TEXT", "CF_OEMTEXT"] {
                assert!(
                    CATALOG.costlier_twin(degraded, source),
                    "{degraded} se guardó pudiendo guardar el Unicode"
                );
            }
            assert!(!CATALOG.costlier_twin("CF_UNICODETEXT", source));
        }
    }

    #[test]
    fn a_screen_capture_costs_its_png_and_not_its_bitmap() {
        assert_eq!(CATALOG.classify(SNIP), Some(Family::Image));
        assert_eq!(CATALOG.preferred_image(SNIP), Some("PNG"));
        assert!(CATALOG.costlier_twin("CF_DIB", SNIP));
        assert!(CATALOG.costlier_twin("CF_DIBV5", SNIP));
        assert!(!CATALOG.costlier_twin("PNG", SNIP));
    }

    #[test]
    fn without_a_png_the_bitmap_with_alpha_wins() {
        let paint = ["DataObject", "CF_BITMAP", "CF_DIB", "CF_DIBV5"];
        assert_eq!(CATALOG.preferred_image(&paint), Some("CF_DIBV5"));
        assert!(CATALOG.costlier_twin("CF_DIB", &paint));
        assert_eq!(CATALOG.classify(&paint), Some(Family::Image));
    }

    #[test]
    fn the_explorer_needs_only_the_paths_and_the_effect() {
        assert_eq!(CATALOG.classify(EXPLORER), Some(Family::Files));
        let kept = kept(EXPLORER);
        assert!(kept.contains(&"CF_HDROP"), "{kept:?}");
        assert!(kept.contains(&"Preferred DropEffect"), "{kept:?}");
        assert_eq!(kept.len(), 2, "y nada más: {kept:?}");
        for partial in ["FileName", "FileNameW"] {
            assert_eq!(CATALOG.decide(partial), Take::Presence, "{partial}");
        }
    }

    #[test]
    fn virtual_files_are_noted_and_never_asked_for() {
        for virtualised in ["FileGroupDescriptorW", "FileContents"] {
            assert_eq!(CATALOG.decide(virtualised), Take::Presence);
        }
    }

    #[test]
    fn the_plainest_copy_there_is_still_an_item() {
        assert_eq!(CATALOG.classify(TERMINAL), Some(Family::Text));
        assert_eq!(kept(TERMINAL), vec!["CF_UNICODETEXT"]);
    }

    #[test]
    fn a_browser_copy_keeps_where_it_came_from() {
        assert_eq!(
            CATALOG.decide("Chromium internal source URL"),
            Take::Payload
        );
        assert_eq!(
            CATALOG.decide("Chromium internal source RFH token"),
            Take::Presence,
            "el testigo interno no es contexto de nadie"
        );
    }

    #[test]
    fn the_unknown_is_only_noted() {
        for unknown in ["ApplicationXYZ", "", "algo/inventado"] {
            assert_eq!(CATALOG.decide(unknown), Take::Presence);
        }
    }

    #[test]
    fn every_measured_source_gets_a_class() {
        for (name, source) in [
            ("Word", WORD),
            ("Excel", EXCEL),
            ("Firefox", FIREFOX),
            ("Chrome", CHROME),
            ("Explorador", EXPLORER),
            ("Recortes", SNIP),
            ("Terminal", TERMINAL),
        ] {
            assert!(CATALOG.classify(source).is_some(), "{name}");
        }
    }

    #[test]
    fn nothing_is_both_wanted_and_wasteful() {
        for id in CATALOG.wanted {
            assert!(
                !CATALOG.wasteful.contains(id),
                "«{id}» está en las dos listas"
            );
        }
    }

    #[test]
    fn everything_that_names_a_family_can_be_copied() {
        for id in CATALOG.text.iter().chain(CATALOG.files) {
            assert_eq!(CATALOG.decide(id), Take::Payload, "«{id}»");
        }
        for id in CATALOG.images_by_preference {
            assert_eq!(CATALOG.decide(id), Take::Payload, "«{id}»");
        }
    }

    #[test]
    fn every_marker_that_is_read_by_value_can_be_read_at_all() {
        for id in CATALOG.denied_when_zero {
            assert_eq!(
                CATALOG.decide(id),
                Take::Payload,
                "«{id}» decide por su valor y nadie pediría sus bytes"
            );
        }
    }

    #[test]
    fn the_preferred_of_each_group_is_one_that_gets_copied() {
        for group in CATALOG.equivalents {
            let first = group.first().expect("un grupo vacío no desempata nada");
            assert_eq!(CATALOG.decide(first), Take::Payload, "«{first}»");
        }
    }
}
