use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};

const VERBATIM: &str = r"\\?\";

pub struct Apartment {
    ours: bool,
}

impl Apartment {
    pub fn enter() -> Self {
        // SAFETY: idempotent when the apartment matches; released in Drop when ours.
        let entered =
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
        Self {
            ours: entered.is_ok(),
        }
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.ours {
            // SAFETY: pairs with the initialisation that this value owns.
            unsafe { CoUninitialize() };
        }
    }
}

pub fn shell_path(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let absolute = path.canonicalize().ok()?;
    let text = absolute.to_str()?;
    Some(std::path::PathBuf::from(
        text.strip_prefix(VERBATIM).unwrap_or(text),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_verbatim_prefix_is_stripped_for_the_shell() {
        let here = shell_path(std::path::Path::new(".")).expect("ruta");
        let text = here.to_string_lossy();
        assert!(!text.starts_with(VERBATIM), "el shell no entiende {text}");
        assert!(here.is_absolute());
    }

    #[test]
    fn a_path_that_is_not_there_has_no_shell_path() {
        let missing = std::path::Path::new(r"C:\no-existe-nada-de-nada");
        assert_eq!(shell_path(missing), None);
    }

    #[test]
    fn entering_twice_is_not_a_problem() {
        let first = Apartment::enter();
        let second = Apartment::enter();
        drop(second);
        drop(first);
    }
}
