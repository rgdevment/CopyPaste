use cp_core::paste::Failure;

pub fn why_not(failure: Failure, english: bool) -> &'static str {
    match failure {
        Failure::NotForeground | Failure::ForegroundTimeout => crate::say::pick_in(
            english,
            "la ventana no volvió al frente; sigue copiado",
            "the window never came back to the front; still copied",
        ),
        Failure::NoKeyboardFocus => crate::say::pick_in(
            english,
            "ahí no había dónde escribir; sigue copiado",
            "there was nowhere to type there; still copied",
        ),
        Failure::TargetGone => crate::say::pick_in(
            english,
            "esa ventana ya no está; sigue copiado",
            "that window is gone; still copied",
        ),
        Failure::SendDenied => crate::say::pick_in(english, denied_es(), denied_en()),
        Failure::InputProtected => crate::say::pick_in(
            english,
            "el sistema protege lo que se escribe; sigue copiado",
            "the system is protecting what is typed; still copied",
        ),
        Failure::TargetElevated => crate::say::pick_in(
            english,
            "esa ventana corre como administrador; sigue copiado",
            "that window runs as administrator; still copied",
        ),
    }
}

#[cfg(target_os = "macos")]
const fn denied_es() -> &'static str {
    "falta el permiso de accesibilidad; sigue copiado"
}

#[cfg(target_os = "macos")]
const fn denied_en() -> &'static str {
    "the accessibility permission is missing; still copied"
}

#[cfg(not(target_os = "macos"))]
const fn denied_es() -> &'static str {
    "el sistema no dejó enviar la pulsación; sigue copiado"
}

#[cfg(not(target_os = "macos"))]
const fn denied_en() -> &'static str {
    "the system refused the keystroke; still copied"
}

#[cfg(test)]
#[path = "excuse_test.rs"]
mod tests;
