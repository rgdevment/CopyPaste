use cp_core::kind::Kind;

pub const UNKNOWN: &str = "?";
pub const AT_A_TIME: usize = 60;

const _: () = assert!(!UNKNOWN.is_empty());
const _: () = assert!(AT_A_TIME >= 1);

pub fn key_of(kind: Option<Kind>, preview: &str) -> String {
    let found = match kind {
        Some(Kind::Link) => crate::link::parts_of(preview)
            .map(|one| one.domain)
            .unwrap_or_default(),
        Some(Kind::Folder) => crate::folder::unit_of(preview),
        Some(Kind::File) => crate::papers::papers_of(preview)
            .map(|one| one.format)
            .unwrap_or_default(),
        _ => return String::new(),
    };
    if found.is_empty() {
        UNKNOWN.to_owned()
    } else {
        found
    }
}

pub fn shown(key: &str) -> bool {
    !key.is_empty() && key != UNKNOWN
}

#[cfg(test)]
#[path = "group_test.rs"]
mod tests;
