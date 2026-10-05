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

#[cfg(any(target_os = "macos", test))]
const MAC_TERMINALS: [&str; 10] = [
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "com.mitchellh.ghostty",
    "net.kovidgoyal.kitty",
    "org.alacritty",
    "com.github.wez.wezterm",
    "dev.warp.Warp-Stable",
    "co.zeit.hyper",
    "org.tabby",
    "com.raphaelamorim.rio",
];

#[cfg(any(target_os = "macos", test))]
const MAC_EDITORS: [&str; 7] = [
    "com.microsoft.VSCode",
    "com.todesktop.230313mzl4w4u92",
    "com.exafunction.windsurf",
    "dev.zed.Zed",
    "com.jetbrains.",
    "com.google.android.studio",
    "com.apple.dt.Xcode",
];

#[cfg(any(target_os = "macos", test))]
pub fn towards_by_bundle(bundle: Option<&str>, hosts_a_terminal: bool) -> Towards {
    let bundle = bundle.unwrap_or_default();
    if MAC_TERMINALS.contains(&bundle) {
        return Towards::Terminal;
    }
    let editor = !bundle.is_empty() && MAC_EDITORS.iter().any(|one| bundle.starts_with(one));
    if hosts_a_terminal && !editor {
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
