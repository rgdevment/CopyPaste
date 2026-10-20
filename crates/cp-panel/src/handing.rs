use crate::FormRow;
use crate::here;
use crate::landing::{Landing, Towards};
use crate::note::note;
use crate::view::{AS_IS, as_is_label, form_of, label_of_form, shorthand_of};
use cp_store::Store;

pub const PLAIN: &str = "plain";

fn landing_for(store: &Store, item: &cp_core::item::Item, id: i64, towards: Towards) -> Landing {
    if towards == Towards::Elsewhere {
        return Landing::anywhere();
    }
    let label = (towards == Towards::Browser)
        .then(|| store.label_of(id).ok().flatten())
        .flatten();
    let content = here::content_of(item, None);
    let files = crate::dragging::files_for(
        item.kind,
        &content.paths,
        content.image,
        label.as_deref(),
        std::time::SystemTime::now(),
    );
    Landing {
        towards,
        files,
        unrecorded: false,
    }
}

pub fn hand_over(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    towards: Towards,
) -> bool {
    let written = placed(store, engine, id, towards, false);
    if written {
        if let Err(why) = store.record_paste(id, crate::app::now_ms()) {
            note(&format!("{id} was pasted and nobody wrote it down: {why}"));
        }
    } else {
        note(&format!(
            "pasting {id}: the write never reached the clipboard"
        ));
    }
    written
}

pub fn dress(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    towards: Towards,
) -> bool {
    placed(store, engine, id, towards, true)
}

fn placed(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    towards: Towards,
    unrecorded: bool,
) -> bool {
    let item = match store.item(id) {
        Ok(Some(item)) => item,
        Ok(None) => {
            note(&format!("placing {id}: it is no longer in the store"));
            return false;
        }
        Err(why) => {
            note(&format!("placing {id}: {why}"));
            return false;
        }
    };
    let landing = Landing {
        unrecorded,
        ..landing_for(store, &item, id, towards)
    };
    let landed = here::to_clipboard(&item, &landing, || starting(engine), || mark(engine));
    short_of(id, landed)
}

pub struct Shelves {
    pub store: std::rc::Rc<Store>,
    pub engine: std::rc::Rc<crate::engine::Engine>,
}

impl crate::dressing::Closet for Shelves {
    fn copied(&self) -> Option<crate::dressing::Copied> {
        self.engine.copied()
    }

    fn copied_again(&self, sequence: i64) {
        self.engine.copied_again(sequence);
    }

    fn place(&self, id: i64, towards: Towards) -> bool {
        dress(&self.store, Some(&self.engine), id, towards)
    }

    fn sequence(&self) -> Option<i64> {
        here::sequence()
    }
}

fn glimpse(rendered: Option<cp_core::paste_as::Rendered>) -> String {
    const SHOWN: usize = 22;
    let text = match rendered {
        Some(cp_core::paste_as::Rendered::Text(text)) => text,
        Some(cp_core::paste_as::Rendered::Jpeg(bytes)) => {
            return format!("{} KB", bytes.len() / 1024);
        }
        None => return String::new(),
    };
    let flat: String = text
        .chars()
        .map(|one| if one.is_control() { ' ' } else { one })
        .collect();
    let trimmed = flat.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.chars().count() <= SHOWN {
        return trimmed;
    }
    let kept: String = trimmed.chars().take(SHOWN).collect();
    format!("{}…", kept.trim_end())
}

pub fn forms_of(store: &Store, id: i64) -> Vec<FormRow> {
    let Ok(Some(item)) = store.item(id) else {
        return Vec::new();
    };
    let ocr = store.ocr_text(id).ok().flatten();
    let content = here::content_of(&item, ocr.as_deref());
    let mut rows = vec![FormRow {
        key: AS_IS.into(),
        label: as_is_label(item.kind).into(),
        preview: glimpse(
            content
                .text
                .as_deref()
                .map(|text| cp_core::paste_as::Rendered::Text(text.to_owned())),
        )
        .into(),
    }];
    rows.extend(
        cp_core::paste_as::forms_for(&content)
            .into_iter()
            .map(|form| {
                let shown = glimpse(cp_core::paste_as::render(form, &content));
                match shorthand_of(form) {
                    Some(short) => FormRow {
                        key: form.as_str().into(),
                        label: shown.into(),
                        preview: short.into(),
                    },
                    None => FormRow {
                        key: form.as_str().into(),
                        label: label_of_form(form).into(),
                        preview: shown.into(),
                    },
                }
            }),
    );
    rows
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    Textless,
    Unmade,
    Failed,
}

impl Refused {
    pub fn said(self) -> &'static str {
        match self {
            Refused::Textless => crate::say::pick(
                "no tiene texto para pegarlo así; elige otra forma",
                "it has no text to paste that way; pick another",
            ),
            Refused::Unmade => crate::say::pick(
                "no se pudo preparar esa forma; elige otra",
                "that form could not be made; pick another",
            ),
            Refused::Failed => crate::app::busy(),
        }
    }
}

pub fn paste_as(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    key: &str,
    towards: Towards,
) -> Result<(), Refused> {
    if key == AS_IS {
        return written_or_not(hand_over(store, engine, id, towards));
    }
    let Ok(Some(item)) = store.item(id) else {
        return Err(Refused::Failed);
    };
    let ocr = store.ocr_text(id).ok().flatten();
    let content = here::content_of(&item, ocr.as_deref());
    let form = if key == PLAIN {
        cp_core::paste_as::plainest(&content)
    } else {
        form_of(key)
    };
    let Some(form) = form else {
        note(&format!(
            "pasting {id} as {key}: there is no such form for it"
        ));
        return Err(if key == PLAIN {
            Refused::Textless
        } else {
            Refused::Unmade
        });
    };
    let Some(rendered) = cp_core::paste_as::render(form, &content) else {
        note(&format!(
            "pasting {id} as {key}: the form gave nothing back"
        ));
        return Err(Refused::Unmade);
    };
    let made = rendered.into_item();
    let written = short_of(
        id,
        here::to_clipboard(
            &made,
            &Landing::anywhere(),
            || starting(engine),
            || mark(engine),
        ),
    );
    if written && let Err(why) = store.record_paste(id, crate::app::now_ms()) {
        note(&format!("{id} was pasted and nobody wrote it down: {why}"));
    }
    written_or_not(written)
}

fn written_or_not(written: bool) -> Result<(), Refused> {
    if written {
        Ok(())
    } else {
        Err(Refused::Failed)
    }
}

pub fn newest(store: &Store, back: usize) -> Option<(cp_store::Listed, bool)> {
    match store.list(&cp_store::Filter::default(), back + 2, None) {
        Ok(page) => {
            let older = page.rows.len() > back + 1;
            page.rows.into_iter().nth(back).map(|row| (row, older))
        }
        Err(why) => {
            note(&format!(
                "the newest in the history could not be read: {why}"
            ));
            None
        }
    }
}

fn short_of(id: i64, landed: here::Landed) -> bool {
    if let here::Landed::Short { placed, wanted } = landed {
        note(&format!(
            "pasting {id}: only {placed} of {wanted} formats fitted on the clipboard"
        ));
    }
    landed != here::Landed::Nothing
}

fn starting(engine: Option<&crate::engine::Engine>) {
    let Some(engine) = engine else {
        return;
    };
    if !engine.writing() {
        note("the start of the clipboard write could not be marked as ours");
    }
}

fn mark(engine: Option<&crate::engine::Engine>) {
    let Some(engine) = engine else {
        return;
    };
    if !engine.ours() {
        note("the clipboard write could not be marked as ours");
    }
}
