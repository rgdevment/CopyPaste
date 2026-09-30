use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

const WHERE_IT_LIVES: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const WHAT_IT_SAYS: &str = "AppsUseLightTheme";

pub fn wants_light() -> Option<bool> {
    let said: u32 = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(WHERE_IT_LIVES)
        .ok()?
        .get_value(WHAT_IT_SAYS)
        .ok()?;
    Some(said == 1)
}

#[cfg(test)]
#[path = "theme_test.rs"]
mod tests;
