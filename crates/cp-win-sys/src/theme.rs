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
mod tests {
    use super::*;

    #[test]
    fn windows_always_has_an_opinion_about_its_own_theme() {
        assert!(wants_light().is_some());
    }

    #[test]
    fn asking_twice_gives_the_same_answer() {
        assert_eq!(wants_light(), wants_light());
    }
}
