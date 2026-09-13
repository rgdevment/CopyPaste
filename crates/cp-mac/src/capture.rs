use crate::formats::CATALOG;
use cp_core::formats::{Family, Take};
use cp_core::item::{Format, Item, Payload};
use cp_core::kind::{self, Kind};
use cp_mac_sys::pasteboard::Pasteboard;

/// Lee todo lo que la fuente ofreció y construye el ítem.
///
/// Devuelve `None` cuando el contenido está marcado como secreto, y esa
/// decisión se toma **antes** de pedir un solo byte.
pub fn capture(pb: &Pasteboard) -> Option<Item> {
    let offered = pb.types();
    let ids: Vec<&str> = offered.iter().map(String::as_str).collect();
    if CATALOG.is_concealed(&ids) {
        return None;
    }

    let family = CATALOG.classify(&ids);
    let cheapest_image = CATALOG.preferred_image(&ids);
    let mut formats: Vec<Format> = Vec::new();

    for id in &ids {
        let canonical = CATALOG.canonical(id);
        if formats.iter().any(|kept| kept.id == canonical) {
            // Los gemelos legados llevan los mismos bytes: guardar los dos
            // duplica el ítem entero.
            continue;
        }
        let payload = match CATALOG.decide(canonical) {
            Take::Never => Payload::Announced { size: None },
            Take::Presence => Payload::Announced { size: None },
            Take::Payload => {
                if is_costlier_twin(canonical, cheapest_image) {
                    // Misma imagen, representación cara: se anota y no se
                    // pide. Medido: 52 veces más grande en el mismo copiado.
                    Payload::Announced { size: None }
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

    // La familia sale del formato; la clase fina, del contenido. Un texto que
    // resulta ser un correo o un color se guarda como tal, que es de lo que
    // vive el filtro por pestañas de la interfaz.
    let kind = refine(family, &formats);
    Some(Item { kind, formats })
}

fn is_costlier_twin(id: &str, cheapest: Option<&str>) -> bool {
    let Some(cheapest) = cheapest else {
        return false;
    };
    CATALOG.images_by_preference.contains(&id) && id != cheapest
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
            Some(match first {
                Some(url) => {
                    let path = url.trim_end_matches('/');
                    let name = path.rsplit('/').next().unwrap_or(path);
                    // Una URL de archivo que termina en barra es una carpeta.
                    kind::classify_file(name, url.ends_with('/'))
                }
                None => Kind::File,
            })
        }
    }
}
