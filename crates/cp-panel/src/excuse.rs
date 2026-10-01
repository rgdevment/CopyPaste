use cp_core::paste::Failure;

pub fn why_not(failure: Failure, english: bool) -> &'static str {
    match failure {
        Failure::NotForeground | Failure::ForegroundTimeout => crate::say::pick_in(
            english,
            "está copiado: la ventana donde ibas a pegar no volvió al frente",
            "it is copied: the window you were pasting into never came back to the front",
        ),
        Failure::NoKeyboardFocus => crate::say::pick_in(
            english,
            "está copiado: ahí no había dónde escribir",
            "it is copied: there was nowhere to type over there",
        ),
        Failure::TargetGone => crate::say::pick_in(
            english,
            "está copiado: la ventana donde ibas a pegar ya no está",
            "it is copied: the window you were pasting into is gone",
        ),
        Failure::SendDenied => crate::say::pick_in(english, denied_es(), denied_en()),
        Failure::InputProtected => crate::say::pick_in(
            english,
            "está copiado: el sistema protege lo que se escribe ahora mismo",
            "it is copied: the system is protecting what is being typed right now",
        ),
        Failure::TargetElevated => crate::say::pick_in(
            english,
            "está copiado: esa ventana corre como administrador y no acepta lo que enviamos",
            "it is copied: that window runs as administrator and refuses what we send",
        ),
    }
}

#[cfg(target_os = "macos")]
const fn denied_es() -> &'static str {
    "está copiado: CopyPaste necesita permiso de accesibilidad para pegar por ti"
}

#[cfg(target_os = "macos")]
const fn denied_en() -> &'static str {
    "it is copied: CopyPaste needs accessibility permission to paste for you"
}

#[cfg(not(target_os = "macos"))]
const fn denied_es() -> &'static str {
    "está copiado: el sistema no dejó enviar la pulsación"
}

#[cfg(not(target_os = "macos"))]
const fn denied_en() -> &'static str {
    "it is copied: the system refused to send the keystroke"
}

#[cfg(test)]
#[path = "excuse_test.rs"]
mod tests;
