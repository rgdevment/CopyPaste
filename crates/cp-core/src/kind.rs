#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Code,
    Json,
    Link,
    Email,
    Phone,
    Color,
    Ip,
    Uuid,
    Image,
    File,
    Folder,
    Audio,
    Video,
    Token,
}

impl Kind {
    pub const ALL: [Kind; 15] = [
        Kind::Text,
        Kind::Code,
        Kind::Json,
        Kind::Link,
        Kind::Email,
        Kind::Phone,
        Kind::Color,
        Kind::Ip,
        Kind::Uuid,
        Kind::Image,
        Kind::File,
        Kind::Folder,
        Kind::Audio,
        Kind::Video,
        Kind::Token,
    ];

    pub fn from_name(name: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.as_str() == name)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Text => "text",
            Kind::Code => "code",
            Kind::Json => "json",
            Kind::Link => "link",
            Kind::Email => "email",
            Kind::Phone => "phone",
            Kind::Color => "color",
            Kind::Ip => "ip",
            Kind::Uuid => "uuid",
            Kind::Image => "image",
            Kind::File => "file",
            Kind::Folder => "folder",
            Kind::Audio => "audio",
            Kind::Video => "video",
            Kind::Token => "token",
        }
    }
}

pub fn classify_text(content: &str) -> Kind {
    let text = content.trim();
    if text.is_empty() {
        return Kind::Text;
    }

    if !text.contains('\n') {
        if crate::token::looks_like(text) {
            return Kind::Token;
        }
        if is_email(text) {
            return Kind::Email;
        }
        if is_url(text) {
            return Kind::Link;
        }
        if is_color(text) {
            return Kind::Color;
        }
        if is_ip(text) {
            return Kind::Ip;
        }
        if is_uuid(text) {
            return Kind::Uuid;
        }
        if is_phone(text) {
            return Kind::Phone;
        }
    }

    if is_json(text) {
        return Kind::Json;
    }
    if looks_like_code(text) {
        return Kind::Code;
    }
    Kind::Text
}

pub fn classify_file(name: &str, is_directory: bool) -> Kind {
    if is_directory {
        return Kind::Folder;
    }
    let extension = name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default();
    const AUDIO: &[&str] = &[
        "mp3", "m4a", "aac", "wav", "aiff", "aif", "flac", "ogg", "opus", "wma",
    ];
    const VIDEO: &[&str] = &[
        "mp4", "mov", "m4v", "avi", "mkv", "webm", "wmv", "mpg", "mpeg", "flv",
    ];
    const IMAGE: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "heic", "heif", "tiff", "tif", "bmp", "svg", "avif",
        "ico",
    ];
    if AUDIO.contains(&extension.as_str()) {
        return Kind::Audio;
    }
    if VIDEO.contains(&extension.as_str()) {
        return Kind::Video;
    }
    if IMAGE.contains(&extension.as_str()) {
        return Kind::Image;
    }
    Kind::File
}

fn is_email(text: &str) -> bool {
    let Some((user, host)) = text.split_once('@') else {
        return false;
    };
    if user.is_empty() {
        return false;
    }
    let Some((label, tld)) = host.rsplit_once('.') else {
        return false;
    };
    let user_ok = user
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c));
    let host_ok = !label.is_empty()
        && label
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));
    let tld_ok = tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic());
    user_ok && host_ok && tld_ok
}

fn is_url(text: &str) -> bool {
    const SCHEMES: &[&str] = &[
        "http://", "https://", "ftp://", "ftps://", "file://", "ssh://", "mailto:",
    ];
    if text.contains(char::is_whitespace) {
        return false;
    }
    SCHEMES
        .iter()
        .any(|scheme| text.len() > scheme.len() && text.to_ascii_lowercase().starts_with(scheme))
}

fn is_color(text: &str) -> bool {
    if let Some(hex) = text.strip_prefix('#') {
        return matches!(hex.len(), 3 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    let lower = text.to_ascii_lowercase();
    for prefix in ["rgba(", "rgb(", "hsla(", "hsl("] {
        if let Some(rest) = lower.strip_prefix(prefix)
            && let Some(inner) = rest.strip_suffix(')')
        {
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
            let expected = if prefix.starts_with("rgba") || prefix.starts_with("hsla") {
                4
            } else {
                3
            };
            return parts.len() == expected
                && parts.iter().all(|part| {
                    let value = part.trim_end_matches('%');
                    !value.is_empty() && value.parse::<f64>().is_ok()
                });
        }
    }
    false
}

fn is_ip(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 4
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.len() <= 3
                && part.chars().all(|c| c.is_ascii_digit())
                && part.parse::<u16>().is_ok_and(|value| value <= 255)
        })
}

fn is_uuid(text: &str) -> bool {
    let groups: Vec<&str> = text.split('-').collect();
    groups.len() == 5
        && [8, 4, 4, 4, 12]
            .iter()
            .zip(&groups)
            .all(|(len, group)| group.len() == *len)
        && groups
            .iter()
            .all(|group| group.chars().all(|c| c.is_ascii_hexdigit()))
}

fn is_phone(text: &str) -> bool {
    let digits = text.chars().filter(char::is_ascii_digit).count();
    if !(7..=15).contains(&digits) {
        return false;
    }
    let shaped = text
        .chars()
        .all(|c| c.is_ascii_digit() || " ()+-.".contains(c));
    shaped && (text.starts_with('+') || text.starts_with('(') || digits >= 9)
}

fn is_json(text: &str) -> bool {
    let bytes = text.as_bytes();
    let (Some(first), Some(last)) = (bytes.first(), bytes.last()) else {
        return false;
    };
    if !matches!((first, last), (b'{', b'}') | (b'[', b']')) {
        return false;
    }
    balanced(text)
}

fn balanced(text: &str) -> bool {
    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    for c in text.chars() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' | '[' => stack.push(c),
            '}' if stack.pop() != Some('{') => return false,
            ']' if stack.pop() != Some('[') => return false,
            _ => {}
        }
    }
    stack.is_empty() && !in_string
}

fn looks_like_code(text: &str) -> bool {
    const MARKERS: &[&str] = &[
        "fn ",
        "def ",
        "class ",
        "function ",
        "const ",
        "let ",
        "var ",
        "import ",
        "#include",
        "public ",
        "private ",
        "SELECT ",
        "INSERT ",
        "UPDATE ",
        "=> ",
        "->",
        "();",
        "{}",
        "</",
        "impl ",
        "struct ",
        "return ",
        "if (",
        "for (",
        "while (",
        "#!/",
    ];
    let hits = MARKERS
        .iter()
        .filter(|marker| text.contains(*marker))
        .count();
    let lines = text.lines().count();
    let indented = text
        .lines()
        .filter(|line| line.starts_with("  ") || line.starts_with('\t'))
        .count();
    hits >= 2 || (hits >= 1 && lines > 1 && indented > 0)
}

#[cfg(test)]
#[path = "kind_test.rs"]
mod tests;

#[cfg(test)]
#[path = "kind_borders_test.rs"]
mod borders;

#[cfg(test)]
#[path = "kind_redundancy_test.rs"]
mod redundancy;

#[cfg(test)]
#[path = "kind_inherited_from_2x_test.rs"]
mod inherited_from_2x;
