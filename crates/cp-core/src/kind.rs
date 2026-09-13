/// Lo que un ítem es, a ojos del producto.
///
/// La 2.x distingue trece tipos y de ellos vive el filtro por pestañas, así
/// que la clasificación fina no es un adorno: perderla sería perder una vista
/// entera de la interfaz. Se hereda la lista y se le añade el código, que la
/// 2.x no reconoce.
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
}

impl Kind {
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
        }
    }
}

/// Afina un texto plano. El orden importa: lo más específico primero, y las
/// formas de una sola línea antes que las que admiten varias.
pub fn classify_text(content: &str) -> Kind {
    let text = content.trim();
    if text.is_empty() {
        return Kind::Text;
    }

    if !text.contains('\n') {
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

/// Un archivo, por su extensión. Una carpeta lo dice quien la lee.
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
        "mp4", "mov", "m4v", "avi", "mkv", "webm", "wmv", "mpg", "mpeg",
    ];
    const IMAGE: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "heic", "heif", "tiff", "tif", "bmp", "svg", "avif",
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
    if user.is_empty() || host.len() < 3 || host.contains('@') {
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
        "http://", "https://", "ftp://", "ftps://", "file://", "ssh://",
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

/// Sin traer un analizador entero: se comprueba que los delimitadores cierran
/// y que las comillas están emparejadas, que es lo que separa un objeto real
/// de un texto que empieza por llave.
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

/// Heurística deliberadamente conservadora: es mejor llamar texto a un
/// fragmento de código que llamar código a una frase.
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
mod tests {
    use super::*;

    #[test]
    fn the_classes_2x_already_recognised() {
        assert_eq!(classify_text("alguien@ejemplo.test"), Kind::Email);
        assert_eq!(classify_text("#FF8800"), Kind::Color);
        assert_eq!(classify_text("rgb(12, 34, 56)"), Kind::Color);
        assert_eq!(classify_text("hsla(120, 50%, 50%, 0.5)"), Kind::Color);
        assert_eq!(classify_text("192.168.1.1"), Kind::Ip);
        assert_eq!(
            classify_text("7ab3f6de-1c4b-4f5e-8a2d-9f0e1b2c3d4e"),
            Kind::Uuid
        );
        assert_eq!(classify_text("+34 600 123 456"), Kind::Phone);
        assert_eq!(classify_text(r#"{"a": 1}"#), Kind::Json);
    }

    #[test]
    fn the_classes_2x_had_a_type_for_but_never_assigned() {
        assert_eq!(classify_text("https://ejemplo.test/ruta?q=1"), Kind::Link);
        assert_eq!(classify_file("cancion.mp3", false), Kind::Audio);
        assert_eq!(classify_file("pelicula.MKV", false), Kind::Video);
        assert_eq!(classify_file("foto.heic", false), Kind::Image);
        assert_eq!(classify_file("Documentos", true), Kind::Folder);
        assert_eq!(classify_file("informe.pdf", false), Kind::File);
    }

    #[test]
    fn code_is_recognised_which_2x_never_did() {
        assert_eq!(
            classify_text("fn main() {\n    println!(\"hola\");\n}"),
            Kind::Code
        );
        assert_eq!(
            classify_text("def suma(a, b):\n    return a + b"),
            Kind::Code
        );
        assert_eq!(
            classify_text("SELECT * FROM tabla WHERE id = 1;\nINSERT INTO otra VALUES (1);"),
            Kind::Code
        );
    }

    #[test]
    fn ordinary_prose_is_never_code() {
        for prose in [
            "Esto es una frase normal y corriente.",
            "Nos vemos mañana si puedes",
            "La reunión es a las cinco, en la sala grande",
            "return", // una palabra suelta no basta
        ] {
            assert_eq!(classify_text(prose), Kind::Text, "«{prose}»");
        }
    }

    #[test]
    fn something_that_only_looks_like_json_is_not_json() {
        for fake in [
            "{esto no es json",
            "[1, 2, 3",
            "{\"sin cerrar\": \"comilla}",
        ] {
            assert_ne!(classify_text(fake), Kind::Json, "«{fake}»");
        }
        assert_eq!(classify_text("[1, 2, 3]"), Kind::Json);
        assert_eq!(classify_text(r#"{"anidado": {"a": [1, 2]}}"#), Kind::Json);
    }

    #[test]
    fn the_edges_of_each_class() {
        assert_eq!(classify_text("#FFF"), Kind::Color);
        assert_eq!(classify_text("#FF8800AA"), Kind::Color);
        assert_ne!(classify_text("#GGGGGG"), Kind::Color);
        assert_ne!(classify_text("256.1.1.1"), Kind::Ip);
        assert_ne!(classify_text("1.2.3"), Kind::Ip);
        assert_ne!(classify_text("no-es-un-uuid-de-verdad"), Kind::Uuid);
        assert_ne!(classify_text("arroba sin@"), Kind::Email);
        assert_ne!(classify_text("12345"), Kind::Phone);
    }

    #[test]
    fn several_lines_never_become_a_single_line_class() {
        assert_eq!(
            classify_text("alguien@ejemplo.test\notra línea"),
            Kind::Text
        );
        assert_eq!(classify_text("#FF8800\n#00FF88"), Kind::Text);
    }

    #[test]
    fn empty_and_blank_are_plain_text() {
        assert_eq!(classify_text(""), Kind::Text);
        assert_eq!(classify_text("   \n  "), Kind::Text);
    }

    #[test]
    fn every_kind_has_a_stable_name_for_the_database() {
        for kind in [
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
        ] {
            assert!(!kind.as_str().is_empty());
            assert_eq!(kind.as_str(), kind.as_str().to_lowercase());
        }
    }
}
