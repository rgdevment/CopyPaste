use crate::Result;
use crate::store::{Clauses, Filter, Order};
use cp_core::search::{Term, holds, terms_of};
use cp_core::typo::{distance, tolerance};
use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, params_from_iter};

pub(crate) const EVERYWHERE: &str = "items.search_text || ' ' || items.search_label || ' ' \
                                     || items.search_app || ' ' || items.search_ocr";

const WORDLESS: &str =
    "(items.search_label = '' AND (items.kind IS 'image' OR items.search_text = ''))";

const WEIGHED: &str = "bm25(items_fts, 1.0, 2.0, 0.3, 0.5)";

const GUESSES: usize = 8;

pub(crate) fn teach(db: &Connection) -> Result<()> {
    db.create_scalar_function(
        "cp_holds",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let text = ctx.get::<String>(0)?;
            let terms = ctx.get::<String>(1)?;
            Ok(holds(&text, &decode(&terms)))
        },
    )?;
    db.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS temp.items_vocab
             USING fts5vocab(main, items_fts, row);",
    )?;
    Ok(())
}

pub(crate) fn fts_expression(terms: &[Term]) -> Option<String> {
    let quoted = |word: &str| format!("\"{}\"", word.replace('"', "\"\""));
    let groups: Vec<String> = terms
        .iter()
        .map(|term| {
            let mut ways = vec![quoted(&term.text)];
            if !term.whole {
                ways.push(quoted(&term.text) + "*");
            }
            ways.extend(term.alike.iter().map(|word| quoted(word)));
            if ways.len() == 1 {
                ways.remove(0)
            } else {
                format!("({})", ways.join(" OR "))
            }
        })
        .collect();
    (!groups.is_empty()).then(|| groups.join(" AND "))
}

pub(crate) fn encode(terms: &[Term]) -> String {
    terms
        .iter()
        .map(|term| {
            let mark = if term.whole { "w" } else { "o" };
            [mark, term.text.as_str()]
                .into_iter()
                .chain(term.alike.iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode(text: &str) -> Vec<Term> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split(' ');
            let whole = words.next()? == "w";
            let text = words.next()?.to_owned();
            Some(Term {
                text,
                whole,
                alike: words.map(str::to_owned).collect(),
            })
        })
        .collect()
}

pub(crate) fn rank_of(filter: &Filter, terms: &[Term]) -> Option<String> {
    if terms.is_empty() {
        return None;
    }
    Some(if filter.order == Order::Recent {
        format!("(CASE WHEN {WORDLESS} THEN 1000000.0 ELSE 0.0 END + {WEIGHED})")
    } else {
        WORDLESS.into()
    })
}

pub(crate) fn terms_for(db: &Connection, filter: &Filter) -> Result<Option<Vec<Term>>> {
    let typed = filter.query.as_deref().map(terms_of).unwrap_or_default();
    if typed.iter().all(|term| tolerance(&term.text) == 0) {
        return Ok(Some(typed));
    }
    let Some(clauses) = Clauses::of(filter, &typed, false, true) else {
        return Ok(Some(typed));
    };
    let sql = format!("SELECT EXISTS (SELECT 1 {})", clauses.source());
    let found: bool = db.query_row(&sql, params_from_iter(clauses.bound.iter()), |row| {
        row.get(0)
    })?;
    if found {
        return Ok(Some(typed));
    }
    guessed(db, typed)
}

fn guessed(db: &Connection, mut terms: Vec<Term>) -> Result<Option<Vec<Term>>> {
    let shortest = terms
        .iter()
        .filter(|term| tolerance(&term.text) > 0)
        .map(|term| term.text.chars().count() - tolerance(&term.text))
        .min()
        .unwrap_or(0);
    let mut near: Vec<Vec<(usize, i64, String)>> = vec![Vec::new(); terms.len()];
    let mut stmt = db.prepare("SELECT term, doc FROM temp.items_vocab WHERE length(term) >= ?")?;
    let rows = stmt.query_map([shortest as i64], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in rows {
        let (word, docs) = row?;
        for (term, found) in terms.iter().zip(near.iter_mut()) {
            if let Some(off) = distance(&term.text, &word, !term.whole)
                && off > 0
            {
                found.push((off, -docs, word.clone()));
            }
        }
    }
    for (term, mut found) in terms.iter_mut().zip(near) {
        found.sort();
        term.alike = found
            .into_iter()
            .take(GUESSES)
            .map(|(_, _, word)| word)
            .collect();
    }
    let guessed = terms.iter().any(|term| !term.alike.is_empty());
    Ok(guessed.then_some(terms))
}

#[cfg(test)]
#[path = "seek_test.rs"]
mod tests;
