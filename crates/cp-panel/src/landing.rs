#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Towards {
    Browser,
    Terminal,
    Elsewhere,
}

pub struct Landing {
    pub towards: Towards,
    pub files: Vec<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Offer {
    Files(Vec<String>),
    Text(String),
    Nothing,
}

impl Landing {
    pub fn anywhere() -> Self {
        Self {
            towards: Towards::Elsewhere,
            files: Vec::new(),
        }
    }

    pub fn offer(&self) -> Offer {
        let named: Vec<String> = self
            .files
            .iter()
            .map(|one| one.to_string_lossy().into_owned())
            .collect();
        if named.is_empty() {
            return Offer::Nothing;
        }
        match self.towards {
            Towards::Browser => Offer::Files(named),
            Towards::Terminal => Offer::Text(quoted(&named)),
            Towards::Elsewhere => Offer::Nothing,
        }
    }
}

const BROWSERS: [&str; 13] = [
    "firefox",
    "chrome",
    "msedge",
    "brave",
    "opera",
    "vivaldi",
    "zen",
    "librewolf",
    "floorp",
    "waterfox",
    "arc",
    "thorium",
    "chromium",
];

const TERMINALS: [&str; 8] = [
    "windowsterminal",
    "openconsole",
    "wezterm-gui",
    "alacritty",
    "mintty",
    "tabby",
    "hyper",
    "conemu64",
];

const TERMINAL_CLASSES: [&str; 3] = [
    "CASCADIA_HOSTING_WINDOW_CLASS",
    "ConsoleWindowClass",
    "org.wezfurlong.wezterm",
];

pub fn towards_of(process: &str, class: &str) -> Towards {
    let process = process.to_ascii_lowercase();
    if TERMINAL_CLASSES.contains(&class) || TERMINALS.contains(&process.as_str()) {
        return Towards::Terminal;
    }
    if BROWSERS.contains(&process.as_str()) {
        return Towards::Browser;
    }
    Towards::Elsewhere
}

pub fn quoted<S: AsRef<str>>(paths: &[S]) -> String {
    paths
        .iter()
        .map(|one| {
            let one = one.as_ref();
            if one.chars().any(char::is_whitespace) {
                format!("\"{one}\"")
            } else {
                one.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
#[path = "landing_test.rs"]
mod tests;
