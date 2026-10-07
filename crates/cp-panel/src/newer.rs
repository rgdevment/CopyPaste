use std::path::Path;

pub const FOUND: &str = "update.json";
pub const PUT_AWAY: &str = "update-put-away";

pub fn waiting(found: &str, here: &str, put_away: Option<&str>) -> Option<String> {
    let said: serde_json::Value = serde_json::from_str(found).ok()?;
    if said.get("from")?.as_str()? != here {
        return None;
    }
    let version = said.get("found")?.as_str()?.trim();
    let away = put_away.map(str::trim);
    (!version.is_empty() && version != here && away != Some(version)).then(|| version.to_owned())
}

pub fn waiting_in(dir: &Path, here: &str) -> Option<String> {
    let found = std::fs::read_to_string(dir.join(FOUND)).ok()?;
    let away = std::fs::read_to_string(dir.join(PUT_AWAY)).ok();
    waiting(&found, here, away.as_deref())
}

pub fn put_away_in(dir: &Path, version: &str) -> std::io::Result<()> {
    std::fs::write(dir.join(PUT_AWAY), version)
}

pub fn line_of(version: &str, english: bool) -> String {
    if english {
        format!("CopyPaste {version} is ready")
    } else {
        format!("CopyPaste {version} está lista")
    }
}

#[cfg(test)]
#[path = "newer_test.rs"]
mod tests;
