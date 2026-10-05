use super::*;

#[test]
fn a_browser_is_known_by_its_process_whatever_its_case() {
    assert_eq!(
        towards_of("firefox", "MozillaWindowClass"),
        Towards::Browser
    );
    assert_eq!(towards_of("Chrome", "Chrome_WidgetWin_1"), Towards::Browser);
    assert_eq!(towards_of("msedge", "Chrome_WidgetWin_1"), Towards::Browser);
}

#[test]
fn an_electron_app_is_not_taken_for_a_browser_by_its_window_class() {
    assert_eq!(
        towards_of("slack", "Chrome_WidgetWin_1"),
        Towards::Elsewhere,
        "Slack, Teams and VS Code draw with Chromium too, but a pasted image there is an image"
    );
}

#[test]
fn a_terminal_is_known_by_its_window_class_or_its_process() {
    assert_eq!(
        towards_of("WindowsTerminal", "CASCADIA_HOSTING_WINDOW_CLASS"),
        Towards::Terminal
    );
    assert_eq!(towards_of("cmd", "ConsoleWindowClass"), Towards::Terminal);
    assert_eq!(
        towards_of("wezterm-gui", "org.wezfurlong.wezterm"),
        Towards::Terminal
    );
    assert_eq!(towards_of("alacritty", "Window Class"), Towards::Terminal);
}

#[test]
fn a_terminal_wins_over_a_browser_and_anything_else_is_left_alone() {
    assert_eq!(
        towards_of("firefox", "ConsoleWindowClass"),
        Towards::Terminal
    );
    assert_eq!(towards_of("WINWORD", "OpusApp"), Towards::Elsewhere);
    assert_eq!(towards_of("", ""), Towards::Elsewhere);
}

#[test]
fn a_path_whose_backslashes_bash_would_read_inside_quotes_is_not_safe() {
    assert!(
        !shell_safe(r"D:\"),
        "a drive's root ends in a backslash, and bash reads it as escaping the closing quote"
    );
    assert!(
        !shell_safe(r"\\server\share\a.png"),
        "bash folds the two leading backslashes of a network path into one"
    );
    assert!(
        shell_safe(r"C:\Temp\a.png"),
        "one backslash before a letter stays one"
    );
}

#[test]
fn a_typographic_quote_is_a_quote_to_powershell() {
    for bad in ['\u{201C}', '\u{201D}', '\u{201E}'] {
        assert!(!shell_safe(&format!(r"C:\informe {bad}final.pdf")));
    }
}

#[test]
fn a_path_is_always_quoted() {
    assert_eq!(quoted(&[r"C:\a\b.png"]), r#""C:\a\b.png""#);
    assert_eq!(quoted(&[r"C:\a b\c.png"]), r#""C:\a b\c.png""#);
    assert_eq!(
        quoted(&[r"C:\x.png", r"C:\y z.png"]),
        r#""C:\x.png" "C:\y z.png""#
    );
    assert_eq!(quoted::<&str>(&[]), "");
}

#[test]
fn a_path_a_shell_would_expand_even_inside_quotes_is_not_safe() {
    assert!(shell_safe(r"C:\Temp\CopyPaste\dragged\a3f1\a3f1.png"));
    assert!(
        shell_safe(r"C:\a b\c&d;e(f).png"),
        "inside double quotes these are only letters"
    );
    for bad in ["$", "`", "%", "!", "\""] {
        assert!(
            !shell_safe(&format!(r"C:\cost {bad}5.png")),
            "{bad} is read by some shell inside quotes"
        );
    }
}

#[test]
fn a_terminal_is_offered_nothing_rather_than_a_path_a_shell_would_expand() {
    assert_eq!(
        landing(Towards::Terminal, &[r"C:\Users\x$y\a.png"]).offer(),
        Offer::Nothing
    );
    assert_eq!(
        landing(Towards::Browser, &[r"C:\Users\x$y\a.png"]).offer(),
        Offer::Files(vec![r"C:\Users\x$y\a.png".to_owned()]),
        "a browser reads the file list, not a command line"
    );
}

fn landing(towards: Towards, files: &[&str]) -> Landing {
    Landing {
        towards,
        files: files.iter().map(std::path::PathBuf::from).collect(),
    }
}

#[test]
fn a_browser_is_offered_the_files_so_each_upload_has_its_own_name() {
    assert_eq!(
        landing(Towards::Browser, &[r"C:\t\a3f1\logo final.png"]).offer(),
        Offer::Files(vec![r"C:\t\a3f1\logo final.png".to_owned()])
    );
}

#[test]
fn a_terminal_is_offered_the_paths_as_text_it_can_paste() {
    assert_eq!(
        landing(Towards::Terminal, &[r"C:\t\a3f1\logo final.png"]).offer(),
        Offer::Text(r#""C:\t\a3f1\logo final.png""#.to_owned())
    );
}

#[test]
fn nothing_is_offered_elsewhere_or_without_a_file() {
    assert_eq!(
        landing(Towards::Elsewhere, &[r"C:\a.png"]).offer(),
        Offer::Nothing
    );
    assert_eq!(landing(Towards::Browser, &[]).offer(), Offer::Nothing);
    assert_eq!(landing(Towards::Terminal, &[]).offer(), Offer::Nothing);
    assert_eq!(Landing::anywhere().offer(), Offer::Nothing);
}
