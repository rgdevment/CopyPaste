use super::*;
use cp_core::kind::Kind;

fn inline(id: &str, bytes: &[u8]) -> Format {
    Format {
        id: id.into(),
        payload: Payload::Inline(bytes.to_vec()),
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-virtual-{}-{name}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    dir
}

#[test]
fn a_descriptor_names_its_files_with_their_sizes_and_folders() {
    let described = described_in(&descriptor_of(&[
        ("report.pdf", Some(1234), false),
        ("attachments", None, true),
        ("attachments\\photo copy.jpg", Some(5 << 32 | 7), false),
    ]));
    assert_eq!(
        described,
        vec![
            Described {
                name: "report.pdf".into(),
                size: Some(1234),
                is_dir: false
            },
            Described {
                name: "attachments".into(),
                size: None,
                is_dir: true
            },
            Described {
                name: "attachments\\photo copy.jpg".into(),
                size: Some(5 << 32 | 7),
                is_dir: false
            },
        ]
    );
}

#[test]
fn the_name_runs_to_the_last_of_its_two_hundred_and_sixty_units() {
    let longest = "n".repeat(NAME_UNITS - 1);
    let described = described_in(&descriptor_of(&[(&longest, None, false)]));
    assert_eq!(described[0].name, longest);
    let accented = "ñ".repeat(200);
    let described = described_in(&descriptor_of(&[(&accented, None, false)]));
    assert_eq!(described[0].name, accented);
}

#[test]
fn a_folder_needs_both_the_flag_and_the_attribute() {
    let mut only_flag = descriptor_of(&[("x", None, true)]);
    only_flag[4 + ATTRIBUTES_AT..4 + ATTRIBUTES_AT + 4].copy_from_slice(&0x20u32.to_le_bytes());
    assert!(
        !described_in(&only_flag)[0].is_dir,
        "FD_ATTRIBUTES on an ordinary file"
    );
    let mut only_attribute = descriptor_of(&[("x", Some(1), false)]);
    only_attribute[4 + ATTRIBUTES_AT..4 + ATTRIBUTES_AT + 4]
        .copy_from_slice(&FILE_ATTRIBUTE_DIRECTORY.to_le_bytes());
    assert!(
        !described_in(&only_attribute)[0].is_dir,
        "without FD_ATTRIBUTES the attribute counts for nothing"
    );
    let mut both_bits = descriptor_of(&[("x", None, true)]);
    both_bits[4 + ATTRIBUTES_AT..4 + ATTRIBUTES_AT + 4]
        .copy_from_slice(&(FILE_ATTRIBUTE_DIRECTORY | 0x20).to_le_bytes());
    assert!(described_in(&both_bits)[0].is_dir);
    let unsized_dir = described_in(&descriptor_of(&[("x", None, true)]));
    assert_eq!(unsized_dir[0].size, None);
}

#[test]
fn the_size_is_read_from_its_own_two_words() {
    let mut swapped = descriptor_of(&[("x", Some(7), false)]);
    swapped[4 + SIZE_HIGH_AT..4 + SIZE_HIGH_AT + 4].copy_from_slice(&3u32.to_le_bytes());
    swapped[4 + SIZE_LOW_AT..4 + SIZE_LOW_AT + 4].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(described_in(&swapped)[0].size, Some(3 << 32));
    assert_eq!(
        described_in(&descriptor_of(&[("x", Some(u32::MAX as u64 + 2), false)]))[0].size,
        Some(u32::MAX as u64 + 2)
    );
}

#[test]
fn a_truncated_or_lying_descriptor_yields_what_it_really_holds() {
    assert!(described_in(&[]).is_empty());
    assert!(described_in(&[0, 0, 0]).is_empty());
    let mut lying = descriptor_of(&[("a.txt", None, false)]);
    lying[..4].copy_from_slice(&9u32.to_le_bytes());
    assert_eq!(described_in(&lying).len(), 1, "says nine, brings one");
    let cut = &descriptor_of(&[("a.txt", None, false)])[..300];
    assert!(described_in(cut).is_empty());
}

#[test]
fn the_contents_ids_count_from_the_descriptor() {
    assert_eq!(contents_id(0), "FileContents#0");
    assert_eq!(index_of("FileContents#12"), Some(12));
    assert_eq!(index_of("FileContents"), None);
    assert_eq!(index_of("FileContents#x"), None);
    assert!(is_virtual(DESCRIPTOR));
    assert!(is_virtual(CONTENTS));
    assert!(is_virtual("FileContents#3"));
    assert!(!is_virtual("CF_HDROP"));
}

#[test]
fn virtual_files_are_offered_only_when_there_is_no_real_drop() {
    assert!(offered_without_a_drop(&[
        DESCRIPTOR,
        CONTENTS,
        "RenPrivateItem"
    ]));
    assert!(
        !offered_without_a_drop(&[DESCRIPTOR, CONTENTS, "CF_HDROP"]),
        "Explorer offers both: the ones on disk win"
    );
    assert!(!offered_without_a_drop(&[DESCRIPTOR]));
    assert!(!offered_without_a_drop(&[CONTENTS]));
}

#[test]
fn a_name_from_the_descriptor_never_escapes_its_folder() {
    assert_eq!(
        relative_path_of("report.pdf"),
        Some(PathBuf::from("report.pdf"))
    );
    assert_eq!(
        relative_path_of("folder\\sub/note.txt"),
        Some(PathBuf::from("folder").join("sub").join("note.txt"))
    );
    for bad in [
        "",
        "..\\x.txt",
        "a\\..\\b",
        ".",
        "C:\\x.txt",
        "\\\\srv\\share",
        "bad<name>.txt",
        "trailing. ",
        "tab\tname",
    ] {
        assert_eq!(relative_path_of(bad), None, "{bad:?}");
    }
}

#[test]
fn materializing_writes_each_file_under_the_item_and_hands_back_the_top_paths() {
    let root = scratch("write");
    let item = Item {
        kind: Some(Kind::File),
        formats: vec![
            inline(
                DESCRIPTOR,
                &descriptor_of(&[
                    ("report.pdf", Some(4), false),
                    ("attachments", None, true),
                    ("attachments\\note.txt", Some(3), false),
                ]),
            ),
            inline(&contents_id(0), b"%PDF"),
            inline(&contents_id(2), b"hello"),
        ],
    };
    let out = materialize(&item, &root).expect("it was written");
    assert!(out.complete);
    assert_eq!(
        out.paths.len(),
        2,
        "report.pdf and attachments: {:?}",
        out.paths
    );
    assert_eq!(std::fs::read(&out.paths[0]).expect("pdf"), b"%PDF");
    assert!(out.paths[1].is_dir());
    assert_eq!(
        std::fs::read(out.paths[1].join("note.txt")).expect("note"),
        b"hello"
    );
    assert!(
        out.paths[0].starts_with(&root),
        "{:?} outside {root:?}",
        out.paths[0]
    );
    let again = materialize(&item, &root).expect("paste twice");
    assert_eq!(
        again.paths, out.paths,
        "the same fingerprint, the same folder"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn what_never_arrived_or_cannot_be_named_leaves_the_paste_incomplete() {
    let root = scratch("incomplete");
    let item = Item {
        kind: Some(Kind::File),
        formats: vec![
            inline(
                DESCRIPTOR,
                &descriptor_of(&[
                    ("big.iso", Some(1 << 40), false),
                    ("..\\outside.txt", Some(1), false),
                    ("ok.txt", Some(2), false),
                ]),
            ),
            Format {
                id: contents_id(0),
                payload: Payload::TooBig { size: 1 << 40 },
            },
            inline(&contents_id(1), b"x"),
            inline(&contents_id(2), b"ok"),
        ],
    };
    let out = materialize(&item, &root).expect("something was written");
    assert!(!out.complete);
    assert_eq!(out.paths.len(), 1);
    assert!(out.paths[0].ends_with("ok.txt"));
    assert!(
        !root.join("outside.txt").exists(),
        "nothing escapes the folder"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn an_item_without_virtual_files_has_nothing_to_materialize() {
    let root = scratch("none");
    assert_eq!(materialize(&Item::plain("hello"), &root), None);
    let empty = Item {
        kind: None,
        formats: vec![inline(DESCRIPTOR, &descriptor_of(&[]))],
    };
    assert_eq!(materialize(&empty, &root), None);
    let announced = Item {
        kind: None,
        formats: vec![Format {
            id: DESCRIPTOR.into(),
            payload: Payload::Announced { size: Some(596) },
        }],
    };
    assert_eq!(materialize(&announced, &root), None);
    assert!(!root.exists(), "with no files, no folder is created");
}
