#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Papers {
    pub format: String,
    pub family: &'static str,
}

const SHEETS: &[&str] = &["xlsx", "xlsm", "xls", "ods", "csv", "tsv"];
const WORDS: &[&str] = &["docx", "docm", "doc", "odt", "rtf"];
const SLIDES: &[&str] = &["pptx", "pptm", "ppt", "odp"];
const PAGES: &[&str] = &["pdf"];

pub fn papers_of(preview: &str) -> Option<Papers> {
    let first = preview.lines().next()?.trim_end();
    let name = first.rsplit(['\\', '/']).next()?;
    let (_, after) = name.rsplit_once('.')?;
    if after.is_empty() || after.len() > 8 || after.contains(' ') {
        return None;
    }
    let folded = after.to_lowercase();
    Some(Papers {
        format: folded.to_uppercase(),
        family: family_of(&folded),
    })
}

pub const FAMILIES: [&str; 5] = ["sheets", "words", "slides", "pages", "plain"];

fn family_of(folded: &str) -> &'static str {
    if SHEETS.contains(&folded) {
        FAMILIES[0]
    } else if WORDS.contains(&folded) {
        FAMILIES[1]
    } else if SLIDES.contains(&folded) {
        FAMILIES[2]
    } else if PAGES.contains(&folded) {
        FAMILIES[3]
    } else {
        FAMILIES[4]
    }
}

#[cfg(test)]
#[path = "papers_test.rs"]
mod tests;
