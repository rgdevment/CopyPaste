#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trouble {
    Unstarted,
    HistoryReplaced,
    HistoryUnopened,
    Undrawn,
    Unwatched,
    Unemptied,
    ClipboardDenied,
    Stopped,
    Unsteady,
    Unanswering,
}

pub const ALL: [Trouble; 10] = [
    Trouble::Unstarted,
    Trouble::HistoryReplaced,
    Trouble::HistoryUnopened,
    Trouble::Undrawn,
    Trouble::Unwatched,
    Trouble::Unemptied,
    Trouble::ClipboardDenied,
    Trouble::Stopped,
    Trouble::Unsteady,
    Trouble::Unanswering,
];

impl Trouble {
    pub fn still_keeping(self) -> bool {
        matches!(self, Self::HistoryReplaced | Self::Unemptied)
    }

    pub fn key(self) -> &'static str {
        match self {
            Trouble::Unstarted => "unstarted",
            Trouble::HistoryReplaced => "history-replaced",
            Trouble::HistoryUnopened => "history-unopened",
            Trouble::Undrawn => "undrawn",
            Trouble::Unwatched => "unwatched",
            Trouble::Unemptied => "unemptied",
            Trouble::ClipboardDenied => "clipboard-denied",
            Trouble::Stopped => "stopped",
            Trouble::Unsteady => "unsteady",
            Trouble::Unanswering => "unanswering",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        ALL.into_iter().find(|one| one.key() == key.trim())
    }

    pub fn worded(self, spanish: bool) -> &'static str {
        let [es, en] = match self {
            Trouble::Unstarted => ["el panel no pudo arrancar", "the panel could not start"],
            Trouble::HistoryReplaced => [
                "el historial estaba dañado y se empezó uno nuevo",
                "the history was damaged and a new one was started",
            ],
            Trouble::HistoryUnopened => [
                "no se pudo abrir el historial",
                "the history could not be opened",
            ],
            Trouble::Undrawn => [
                "no se pudo dibujar el panel",
                "the panel could not be drawn",
            ],
            Trouble::Unwatched => [
                "nada está vigilando el portapapeles",
                "nothing is watching the clipboard",
            ],
            Trouble::Unemptied => [
                "no se pudo vaciar el historial",
                "the history could not be emptied",
            ],
            Trouble::ClipboardDenied => [
                "el sistema no deja que CopyPaste lea el portapapeles",
                "the system does not let CopyPaste read the clipboard",
            ],
            Trouble::Stopped => [
                "el panel dejó de vigilar el portapapeles",
                "the panel stopped watching the clipboard",
            ],
            Trouble::Unsteady => [
                "el panel no se mantiene abierto; reinicia CopyPaste",
                "the panel will not stay up; restart CopyPaste",
            ],
            Trouble::Unanswering => ["el panel no responde", "the panel is not answering"],
        };
        if spanish { es } else { en }
    }
}

#[cfg(test)]
#[path = "trouble_test.rs"]
mod tests;
