use std::sync::{Arc, Mutex, PoisonError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub english: bool,
    pub theme: cp_config::Theme,
    pub hides: bool,
    pub look: Look,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    pub size: cp_config::TextSize,
    pub density: cp_config::Density,
    pub accent: cp_config::Accent,
    pub font: String,
    pub mono: String,
}

impl Default for Look {
    fn default() -> Self {
        let fonts = cp_config::look::fonts_here();
        Self {
            size: cp_config::TextSize::default(),
            density: cp_config::Density::default(),
            accent: cp_config::Accent::default(),
            font: fonts.text.family.to_owned(),
            mono: fonts.code.family.to_owned(),
        }
    }
}

pub fn look_of(config: &cp_config::Config, fonts: &cp_config::look::Choices) -> Look {
    Look {
        size: config.text_size,
        density: config.density,
        accent: config.accent,
        font: fonts.text_family(config.font.as_deref()).to_owned(),
        mono: fonts.code_family(config.code_font.as_deref()).to_owned(),
    }
}

impl Kept {
    pub fn light(&self, the_system_is_light: bool) -> bool {
        light_for(self.theme, the_system_is_light)
    }
}

#[derive(Clone)]
pub struct Shelf {
    kept: Arc<Mutex<Kept>>,
    fonts: Arc<cp_config::look::Choices>,
}

impl Shelf {
    pub fn new(kept: Kept, fonts: cp_config::look::Choices) -> Self {
        Self {
            kept: Arc::new(Mutex::new(kept)),
            fonts: Arc::new(fonts),
        }
    }

    pub fn get(&self) -> Kept {
        self.kept
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn set(&self, kept: Kept) {
        *self.kept.lock().unwrap_or_else(PoisonError::into_inner) = kept;
    }

    pub fn renew(&self) -> Kept {
        self.renew_from(crate::here::data_dir().as_deref())
    }

    pub fn renew_from(&self, dir: Option<&std::path::Path>) -> Kept {
        let kept = read_from(dir, &self.fonts);
        self.set(kept.clone());
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

pub fn resolve(config: Option<&cp_config::Config>, fonts: &cp_config::look::Choices) -> Kept {
    Kept {
        english: crate::say::english_for(config.and_then(|kept| kept.locale.as_deref())),
        theme: config.map_or(cp_config::Theme::System, |kept| kept.theme),
        hides: config.is_none_or(|kept| kept.hides_when_left),
        look: look_of(config.unwrap_or(&cp_config::Config::default()), fonts),
    }
}

pub fn read(fonts: &cp_config::look::Choices) -> Kept {
    read_from(crate::here::data_dir().as_deref(), fonts)
}

pub fn read_from(dir: Option<&std::path::Path>, fonts: &cp_config::look::Choices) -> Kept {
    let config = dir.and_then(|dir| cp_config::read(&cp_config::at(dir)).ok());
    resolve(config.as_ref(), fonts)
}

#[cfg(test)]
#[path = "kept_test.rs"]
mod tests;
