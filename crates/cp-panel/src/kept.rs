use std::sync::{Arc, Mutex, PoisonError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    pub english: bool,
    pub theme: cp_config::Theme,
    pub hides: bool,
}

impl Kept {
    pub fn light(&self, the_system_is_light: bool) -> bool {
        light_for(self.theme, the_system_is_light)
    }
}

#[derive(Clone)]
pub struct Shelf(Arc<Mutex<Kept>>);

impl Shelf {
    pub fn new(kept: Kept) -> Self {
        Self(Arc::new(Mutex::new(kept)))
    }

    pub fn get(&self) -> Kept {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn set(&self, kept: Kept) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = kept;
    }

    pub fn renew(&self) -> Kept {
        self.renew_from(crate::here::data_dir().as_deref())
    }

    pub fn renew_from(&self, dir: Option<&std::path::Path>) -> Kept {
        let kept = read_from(dir);
        self.set(kept);
        kept
    }
}

pub fn light_for(asked: cp_config::Theme, the_system_is_light: bool) -> bool {
    match asked {
        cp_config::Theme::Light => true,
        cp_config::Theme::Dark => false,
        cp_config::Theme::System => the_system_is_light,
    }
}

pub fn resolve(config: Option<&cp_config::Config>) -> Kept {
    Kept {
        english: crate::say::english_for(config.and_then(|kept| kept.locale.as_deref())),
        theme: config.map_or(cp_config::Theme::System, |kept| kept.theme),
        hides: config.is_none_or(|kept| kept.hides_when_left),
    }
}

pub fn read() -> Kept {
    read_from(crate::here::data_dir().as_deref())
}

pub fn read_from(dir: Option<&std::path::Path>) -> Kept {
    let config = dir.and_then(|dir| cp_config::read(&cp_config::at(dir)).ok());
    resolve(config.as_ref())
}

#[cfg(test)]
#[path = "kept_test.rs"]
mod tests;
