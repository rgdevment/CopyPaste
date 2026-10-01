#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Inline(Vec<u8>),
    Blob(Vec<u8>),
    TooBig { size: usize },
    Announced { size: Option<usize> },
    Absent,
}

pub const INLINE_UP_TO: usize = 64 * 1024;
pub const BLOB_UP_TO: usize = 64 * 1024 * 1024;

const _: () = assert!(INLINE_UP_TO < BLOB_UP_TO);
const _: () = assert!(INLINE_UP_TO.is_power_of_two() && BLOB_UP_TO.is_power_of_two());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Row,
    Blob,
    Refused,
}

pub fn placement(size: usize) -> Placement {
    match size {
        0..=INLINE_UP_TO => Placement::Row,
        size if size <= BLOB_UP_TO => Placement::Blob,
        _ => Placement::Refused,
    }
}

impl Payload {
    pub fn stored(bytes: Vec<u8>) -> Self {
        match placement(bytes.len()) {
            Placement::Row => Payload::Inline(bytes),
            Placement::Blob => Payload::Blob(bytes),
            Placement::Refused => Payload::TooBig { size: bytes.len() },
        }
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.len()),
            Payload::TooBig { size } => Some(*size),
            Payload::Announced { size } => *size,
            Payload::Absent => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    pub id: String,
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: Option<crate::kind::Kind>,
    pub formats: Vec<Format>,
}

pub const SYNTHETIC_TEXT: &str = "text/plain";

pub const SYNTHETIC_IMAGE: &str = "image/png";

pub const SYNTHETIC_JPEG: &str = "image/jpeg";

impl Item {
    pub fn plain(text: &str) -> Self {
        Self {
            kind: None,
            formats: vec![Format {
                id: SYNTHETIC_TEXT.into(),
                payload: Payload::Inline(text.as_bytes().to_vec()),
            }],
        }
    }

    pub fn fingerprint(&self) -> u64 {
        let mut with_bytes: Vec<(&str, &[u8])> = self
            .formats
            .iter()
            .filter_map(|format| match &format.payload {
                Payload::Inline(bytes) | Payload::Blob(bytes) => {
                    Some((format.id.as_str(), bytes.as_slice()))
                }
                _ => None,
            })
            .collect();
        if with_bytes
            .iter()
            .any(|(id, _)| crate::identity::bears_identity(id))
        {
            with_bytes.retain(|(id, _)| crate::identity::bears_identity(id));
        }
        with_bytes.sort_by(|a, b| a.0.cmp(b.0));
        let mut mixed: Vec<u8> = Vec::new();
        for (id, bytes) in with_bytes {
            mixed.extend_from_slice(id.as_bytes());
            mixed.push(0);
            mixed.extend_from_slice(&crate::identity::stable(id, bytes));
            mixed.push(0);
        }
        let mut unread: Vec<(&str, u64)> = self
            .formats
            .iter()
            .filter_map(|format| match &format.payload {
                Payload::TooBig { size } => Some((format.id.as_str(), *size as u64)),
                _ => None,
            })
            .collect();
        unread.sort_by(|a, b| a.0.cmp(b.0));
        for (id, size) in unread {
            mixed.extend_from_slice(id.as_bytes());
            mixed.push(0);
            mixed.extend_from_slice(&size.to_le_bytes());
            mixed.push(0);
        }
        crate::hash::content_hash(&mixed)
    }

    pub fn is_comparable(&self) -> bool {
        self.formats
            .iter()
            .any(|format| matches!(format.payload, Payload::Inline(_) | Payload::Blob(_)))
    }

    pub fn needs_blob_store(&self) -> bool {
        self.oversized_format().is_some()
    }

    pub fn oversized_format(&self) -> Option<(String, usize)> {
        self.formats.iter().find_map(|one| match &one.payload {
            Payload::Blob(bytes) => Some((one.id.clone(), bytes.len())),
            _ => None,
        })
    }

    pub fn format(&self, id: &str) -> Option<&Format> {
        self.formats.iter().find(|one| one.id == id)
    }

    pub fn stored_bytes(&self) -> usize {
        self.formats
            .iter()
            .filter_map(|one| match &one.payload {
                Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.len()),
                _ => None,
            })
            .sum()
    }
}

#[cfg(test)]
#[path = "item_test.rs"]
mod tests;

#[cfg(test)]
#[path = "item_properties_test.rs"]
mod properties;
