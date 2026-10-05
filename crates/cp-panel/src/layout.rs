use cp_core::kind::Kind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Everything,
    Json,
    Image,
    Link,
    Video,
    Audio,
    Folder,
    Papers,
}

impl Layout {
    #[cfg(test)]
    pub const ALL: [Layout; 8] = [
        Layout::Everything,
        Layout::Json,
        Layout::Image,
        Layout::Link,
        Layout::Video,
        Layout::Audio,
        Layout::Folder,
        Layout::Papers,
    ];

    pub fn shows_cards(self) -> bool {
        matches!(
            self,
            Layout::Everything | Layout::Json | Layout::Link | Layout::Image
        )
    }

    pub fn groups(self) -> bool {
        matches!(self, Layout::Link | Layout::Folder | Layout::Papers)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Layout::Everything => "everything",
            Layout::Json => "json",
            Layout::Image => "image",
            Layout::Link => "link",
            Layout::Video => "video",
            Layout::Audio => "audio",
            Layout::Folder => "folder",
            Layout::Papers => "papers",
        }
    }
}

pub fn layout_for(kinds: &[Kind]) -> Layout {
    let mut only = None;
    for kind in kinds {
        match only {
            None => only = Some(*kind),
            Some(seen) if seen == *kind => {}
            Some(_) => return Layout::Everything,
        }
    }
    only.map_or(Layout::Everything, of_kind)
}

fn of_kind(kind: Kind) -> Layout {
    match kind {
        Kind::Json => Layout::Json,
        Kind::Image => Layout::Image,
        Kind::Link => Layout::Link,
        Kind::Video => Layout::Video,
        Kind::Audio => Layout::Audio,
        Kind::Folder => Layout::Folder,
        Kind::File => Layout::Papers,
        Kind::Text
        | Kind::Code
        | Kind::Email
        | Kind::Phone
        | Kind::Color
        | Kind::Ip
        | Kind::Uuid
        | Kind::Token => Layout::Everything,
    }
}

#[cfg(test)]
#[path = "layout_test.rs"]
mod tests;
