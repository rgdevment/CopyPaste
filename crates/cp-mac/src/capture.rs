use crate::formats::CATALOG;
use cp_core::formats::Take;
use cp_core::item::{Format, Item, Payload};
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

    let kind = CATALOG.classify(&ids);
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

    Some(Item { kind, formats })
}

fn is_costlier_twin(id: &str, cheapest: Option<&str>) -> bool {
    let Some(cheapest) = cheapest else {
        return false;
    };
    CATALOG.images_by_preference.contains(&id) && id != cheapest
}
