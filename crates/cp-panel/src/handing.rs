use crate::FormRow;
use crate::here;
use crate::landing::{Landing, Towards};
use crate::note::note;
use crate::view::{AS_IS, as_is_label, form_of, label_of_form, shorthand_of};
use cp_store::Store;

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
    Landing { towards, files }
}

pub fn hand_over(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    towards: Towards,
) -> bool {
    let item = match store.item(id) {
        Ok(Some(item)) => item,
        Ok(None) => {
            note(&format!("pasting {id}: it is no longer in the store"));
            return false;
        }
        Err(why) => {
            note(&format!("pasting {id}: {why}"));
            return false;
        }
    };
    let landing = landing_for(store, &item, id, towards);
    let landed = here::to_clipboard(&item, &landing, || starting(engine), || mark(engine));
    let written = short_of(id, landed);
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

pub fn paste_as(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    key: &str,
    towards: Towards,
) -> bool {
    if key == AS_IS {
        return hand_over(store, engine, id, towards);
    }
    let Some(form) = form_of(key) else {
        note(&format!("a form nobody knows: {key}"));
        return false;
    };
    let Ok(Some(item)) = store.item(id) else {
        return false;
    };
    let ocr = store.ocr_text(id).ok().flatten();
    let content = here::content_of(&item, ocr.as_deref());
    let Some(rendered) = cp_core::paste_as::render(form, &content) else {
        note(&format!(
            "pasting {id} as {key}: the form gave nothing back"
        ));
        return false;
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
    written
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
