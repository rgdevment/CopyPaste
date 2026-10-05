use super::*;

#[test]
fn the_name_is_what_a_person_recognises() {
    assert_eq!(
        stem_of(r"C:\Program Files\Google\Chrome\chrome.exe"),
        "chrome"
    );
    assert_eq!(stem_of(r"C:\Windows\explorer.exe"), "explorer");
    assert_eq!(stem_of("WINWORD.EXE"), "WINWORD");
}

#[test]
fn what_is_in_front_is_never_ourselves() {
    if let Some(named) = in_front() {
        assert!(!named.is_empty());
        assert_ne!(Some(named), name_of(std::process::id()));
    }
}

#[test]
fn a_name_without_a_folder_or_an_extension_still_works() {
    assert_eq!(stem_of("notepad"), "notepad");
    assert_eq!(stem_of(""), "");
}

#[test]
fn a_folder_with_a_dot_does_not_eat_the_name() {
    assert_eq!(stem_of(r"C:\apps\v1.2\editor.exe"), "editor");
    assert_eq!(stem_of(r"C:\apps\v1.2\editor"), "editor");
}

#[test]
fn both_separators_are_understood() {
    assert_eq!(stem_of("C:/Windows/System32/cmd.exe"), "cmd");
}

#[test]
fn this_very_process_can_be_named() {
    let mine = std::process::id();
    assert!(name_of(mine).is_some());
}

#[test]
fn a_window_that_is_not_there_describes_nothing() {
    assert_eq!(described(0), None);
}

#[test]
fn the_desktop_window_has_the_class_windows_gives_it() {
    let desktop = unsafe { windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow() };
    assert_eq!(class_of(desktop).as_deref(), Some("#32769"));
}

#[test]
fn a_console_host_is_known_by_either_of_its_two_names() {
    assert!(is_console_host("conhost.exe"));
    assert!(is_console_host("OpenConsole.exe"));
    assert!(!is_console_host("cmd.exe"));
    assert!(!is_console_host("conhost"));
}

#[test]
fn only_a_headless_console_host_is_a_pseudoconsole() {
    assert!(is_headless(
        r#""C:\Program Files\WezTerm\OpenConsole.exe" --headless --width 72 --height 19 --signal 0x9e8"#
    ));
    assert!(
        !is_headless(r"\??\C:\WINDOWS\system32\conhost.exe 0x4"),
        "the console a console program is given is not one a terminal draws"
    );
    assert!(!is_headless("conhost.exe --headlessly"));
}

#[test]
fn no_window_hosts_a_pseudoconsole() {
    assert!(!hosts_a_pseudoconsole(0));
}

#[test]
fn a_process_with_no_children_hosts_nothing() {
    assert!(children_of(u32::MAX).is_empty());
}

#[test]
fn this_very_process_has_a_command_line() {
    let line = command_line_of(std::process::id()).expect("readable");
    assert!(!line.is_empty());
}

#[test]
fn a_process_that_opens_a_pseudoconsole_is_found_hosting_one() {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::{COORD, ClosePseudoConsole, CreatePseudoConsole};
    use windows::Win32::System::Pipes::CreatePipe;

    let mine = std::process::id();
    assert!(
        !process_hosts_a_pseudoconsole(mine),
        "nothing is hosted before the pseudoconsole opens"
    );

    let (mut input_read, mut input_write) = (HANDLE::default(), HANDLE::default());
    let (mut output_read, mut output_write) = (HANDLE::default(), HANDLE::default());
    unsafe { CreatePipe(&mut input_read, &mut input_write, None, 0) }.expect("an input pipe");
    unsafe { CreatePipe(&mut output_read, &mut output_write, None, 0) }.expect("an output pipe");
    let console =
        unsafe { CreatePseudoConsole(COORD { X: 80, Y: 25 }, input_read, output_write, 0) }
            .expect("a pseudoconsole");

    let hosted = (0..50).any(|_| {
        let found = process_hosts_a_pseudoconsole(mine);
        if !found {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        found
    });

    unsafe { ClosePseudoConsole(console) };
    for pipe in [input_read, input_write, output_read, output_write] {
        let _ = unsafe { CloseHandle(pipe) };
    }
    assert!(
        hosted,
        "Windows starts a headless console host as a child of whoever asked"
    );
}

#[test]
fn a_child_just_started_is_listed_with_its_name() {
    let mut child = std::process::Command::new("cmd.exe")
        .args(["/c", "ping", "-n", "3", "127.0.0.1"])
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("cmd starts");
    let found = children_of(std::process::id());
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        found
            .iter()
            .any(|(pid, exe)| *pid == child.id() && exe.eq_ignore_ascii_case("cmd.exe"))
    );
}
