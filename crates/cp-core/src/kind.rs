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
            "return",
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

#[cfg(test)]
mod borders {
    use super::*;

    #[test]
    fn every_class_keeps_the_name_the_database_stores() {
        for (kind, name) in [
            (Kind::Text, "text"),
            (Kind::Code, "code"),
            (Kind::Json, "json"),
            (Kind::Link, "link"),
            (Kind::Email, "email"),
            (Kind::Phone, "phone"),
            (Kind::Color, "color"),
            (Kind::Ip, "ip"),
            (Kind::Uuid, "uuid"),
            (Kind::Image, "image"),
            (Kind::File, "file"),
            (Kind::Folder, "folder"),
            (Kind::Audio, "audio"),
            (Kind::Video, "video"),
        ] {
            assert_eq!(kind.as_str(), name, "el nombre viaja a la columna kind");
        }
    }

    #[test]
    fn an_address_needs_every_piece_at_once() {
        assert!(is_email("a@b.co"));
        assert!(!is_email("@ejemplo.test"), "sin usuario");
        assert!(!is_email("a@b@c.co"), "dos arrobas");
        assert!(!is_email("a@.co"), "sin nombre de dominio");
        assert!(!is_email("a@b.c"), "dominio de primer nivel de una letra");
        assert!(!is_email("a@b.c1"), "dominio de primer nivel con cifra");
        assert!(!is_email("a b@c.co"), "espacio en el usuario");
        assert!(!is_email("a@b c.co"), "espacio en el dominio");
        assert!(!is_email("a@bc"), "sin punto");
    }

    #[test]
    fn a_scheme_on_its_own_is_not_an_address() {
        assert!(is_url("http://a"));
        assert!(!is_url("http://"), "el esquema entero y nada más");
        assert!(!is_url("https://"));
    }

    #[test]
    fn a_colour_needs_length_and_digits_at_once() {
        assert!(is_color("#FF8800"));
        assert!(
            !is_color("#GGG"),
            "tres caracteres que no son hexadecimales"
        );
        assert!(!is_color("rgb(1, 2)"), "le falta una componente");
        assert!(
            !is_color("rgb(a, b, c)"),
            "tres componentes que no son números"
        );
    }

    #[test]
    fn a_uuid_needs_shape_and_digits_at_once() {
        assert!(is_uuid("6ba7b810-9dad-11d1-80b4-00c04fd430c8"));
        assert!(!is_uuid("1-2-3-4-5"), "cinco grupos de largo equivocado");
        assert!(
            !is_uuid("zzzzzzzz-9dad-11d1-80b4-00c04fd430c8"),
            "el largo correcto con caracteres que no son hexadecimales"
        );
    }

    #[test]
    fn a_number_needs_shape_and_a_reason_to_be_a_phone() {
        assert!(is_phone("+1234567"), "el prefijo internacional basta");
        assert!(is_phone("(123) 4567"), "el paréntesis de área basta");
        assert!(is_phone("123456789"), "nueve cifras bastan por sí solas");
        assert!(
            !is_phone("1234567"),
            "siete cifras sueltas no son un teléfono"
        );
        assert!(!is_phone("(123) 456-7890 ñ"), "una letra rompe la forma");
    }

    #[test]
    fn nothing_at_all_is_not_an_object() {
        assert!(!is_json(""), "sin un primer carácter no hay delimitador");
        assert!(is_json("{}"));
    }

    #[test]
    fn a_quote_inside_a_string_does_not_close_it() {
        assert!(balanced(r#"{"\""}"#), "la comilla escapada sigue dentro");
        assert!(!balanced(r#"{"a": {}"#), "queda una llave sin cerrar");
        assert!(!balanced(r#"{"a"#), "la cadena se quedó abierta");
    }

    #[test]
    fn one_marker_alone_is_not_code() {
        assert!(looks_like_code("const a = 1;\n  return a;"), "dos marcas");
        assert!(!looks_like_code("const"), "ni una marca completa");
        assert!(
            !looks_like_code("const x"),
            "una marca, una línea, sin sangrar"
        );
        assert!(
            !looks_like_code("  const x"),
            "una marca sangrada, pero de una sola línea"
        );
        assert!(
            !looks_like_code("const x\nmás texto"),
            "una marca en dos líneas, ninguna sangrada"
        );
    }
}

#[cfg(test)]
mod redundancy {
    use super::*;
    use proptest::prelude::*;

    fn with_the_old_guards(text: &str) -> bool {
        let Some((user, host)) = text.split_once('@') else {
            return false;
        };
        if user.is_empty() || host.len() < 3 || host.contains('@') {
            return false;
        }
        is_email(text)
    }

    proptest! {
        #[test]
        fn the_guards_that_were_here_were_redundant(text in ".{0,40}") {
            prop_assert_eq!(is_email(&text), with_the_old_guards(&text));
        }

        #[test]
        fn the_guards_were_redundant_for_addresses_too(
            user in "[a-z@._%+-]{0,8}",
            host in "[a-z@.-]{0,8}",
        ) {
            let text = format!("{user}@{host}");
            prop_assert_eq!(is_email(&text), with_the_old_guards(&text));
        }
    }
}

#[cfg(test)]
mod inherited_from_2x {
    use super::*;

    #[test]
    fn the_three_the_rewrite_had_dropped() {
        assert_eq!(classify_file("video.flv", false), Kind::Video);
        assert_eq!(classify_file("favicon.ico", false), Kind::Image);
        assert_eq!(classify_text("mailto:alguien@ejemplo.test"), Kind::Link);
    }

    #[test]
    fn what_the_rewrite_added_survives() {
        for (name, kind) in [
            ("captura.heic", Kind::Image),
            ("captura.avif", Kind::Image),
            ("pelicula.m4v", Kind::Video),
            ("pelicula.mpeg", Kind::Video),
            ("audio.opus", Kind::Audio),
            ("audio.aiff", Kind::Audio),
        ] {
            assert_eq!(classify_file(name, false), kind, "«{name}»");
        }
    }

    #[test]
    fn a_bare_mail_scheme_is_not_a_link() {
        assert_eq!(classify_text("mailto:"), Kind::Text);
        assert_eq!(classify_text("MAILTO:alguien@ejemplo.test"), Kind::Link);
    }

    #[test]
    fn an_address_without_the_scheme_is_still_an_address() {
        assert_eq!(classify_text("alguien@ejemplo.test"), Kind::Email);
    }

    #[test]
    fn a_windows_path_is_classified_by_its_last_segment() {
        for (path, kind) in [
            (r"C:\Users\ana\Imagenes\foto.PNG", Kind::Image),
            (r"C:\fotos.2024\captura.jpg", Kind::Image),
            (r"C:\version.1.2\programa", Kind::File),
            (r"\servidor\compartido\clip.MKV", Kind::Video),
            (r"C:\sin_extension", Kind::File),
        ] {
            assert_eq!(classify_file(path, false), kind, "«{path}»");
        }
    }

    #[test]
    fn a_folder_named_like_a_file_is_still_a_folder() {
        assert_eq!(classify_file(r"C:\copias\respaldo.zip", true), Kind::Folder);
        assert_eq!(classify_file("fotos.png", true), Kind::Folder);
    }

    #[test]
    fn a_name_that_is_only_a_dot_has_no_extension() {
        assert_eq!(classify_file(".png", false), Kind::Image);
        assert_eq!(classify_file(".", false), Kind::File);
        assert_eq!(classify_file("", false), Kind::File);
    }

    #[test]
    fn a_local_windows_path_is_not_a_link() {
        assert_eq!(classify_text(r"C:\Users\ana\documento.txt"), Kind::Text);
    }
}
