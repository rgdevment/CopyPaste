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
            Towards::Terminal if named.iter().all(|one| shell_safe(one)) => {
                Offer::Text(quoted(&named))
            }
            Towards::Terminal => Offer::Nothing,
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

const TERMINALS: [&str; 12] = [
    "windowsterminal",
    "openconsole",
    "wezterm-gui",
    "alacritty",
    "mintty",
    "tabby",
    "hyper",
    "conemu64",
    "conemu",
    "putty",
    "kitty",
    "mobaxterm",
];

const TERMINAL_CLASSES: [&str; 3] = [
    "CASCADIA_HOSTING_WINDOW_CLASS",
    "ConsoleWindowClass",
    "org.wezfurlong.wezterm",
];

const EDITORS: [&str; 18] = [
    "code",
    "code - insiders",
    "cursor",
    "windsurf",
    "zed",
    "devenv",
    "fleet",
    "idea64",
    "pycharm64",
    "webstorm64",
    "rider64",
    "clion64",
    "goland64",
    "phpstorm64",
    "rubymine64",
    "datagrip64",
    "rustrover64",
    "studio64",
];

pub fn towards_of(process: &str, class: &str, hosts_a_pseudoconsole: bool) -> Towards {
    let process = process.to_ascii_lowercase();
    if TERMINAL_CLASSES.contains(&class) || TERMINALS.contains(&process.as_str()) {
        return Towards::Terminal;
    }
    if BROWSERS.contains(&process.as_str()) {
        return Towards::Browser;
    }
    if hosts_a_pseudoconsole && !EDITORS.contains(&process.as_str()) {
        return Towards::Terminal;
    }
    Towards::Elsewhere
}

const EXPANDED_INSIDE_QUOTES: [char; 8] =
    ['$', '`', '%', '!', '"', '\u{201C}', '\u{201D}', '\u{201E}'];

pub fn shell_safe(path: &str) -> bool {
    !path.contains(EXPANDED_INSIDE_QUOTES) && !path.contains(r"\\") && !path.ends_with('\\')
}

pub fn quoted<S: AsRef<str>>(paths: &[S]) -> String {
    paths
        .iter()
        .map(|one| format!("\"{}\"", one.as_ref()))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
#[path = "landing_test.rs"]
mod tests;
