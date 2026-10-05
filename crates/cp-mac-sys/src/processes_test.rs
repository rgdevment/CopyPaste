use super::*;
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
fn this_very_process_knows_its_parent() {
    assert_eq!(
        parent_of(std::process::id() as i32),
        Some(std::os::unix::process::parent_id() as i32)
    );
}

#[test]
fn a_child_just_started_is_found_among_the_children() {
    let mut child = Command::new("/bin/sleep")
        .arg("5")
        .spawn()
        .expect("sleep starts");
    let found = children_of(std::process::id() as i32);
    let _ = child.kill();
    let _ = child.wait();
    assert!(found.contains(&(child.id() as i32)));
}

#[test]
fn nobody_hosts_nothing() {
    assert!(children_of(0).is_empty());
    assert!(!hosts_a_terminal(0));
    assert!(!has_a_terminal(-1));
}

#[test]
fn a_process_that_gives_its_child_a_pseudoterminal_hosts_a_terminal() {
    if has_a_terminal(std::process::id() as i32) {
        eprintln!("skipped: run from a terminal, every child inherits it");
        return;
    }
    let mut script = Command::new("/usr/bin/script")
        .args(["-q", "/dev/null", "/bin/sleep", "5"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("script starts");
    std::thread::sleep(Duration::from_millis(500));
    let alive = script.try_wait().ok().flatten().is_none();
    let hosts = hosts_a_terminal(std::process::id() as i32);
    let _ = script.kill();
    let _ = script.wait();
    if !alive {
        eprintln!("skipped: script would not run without a terminal of its own");
        return;
    }
    assert!(
        hosts,
        "script opened a pseudoterminal for sleep, one level down"
    );
}

#[test]
fn a_process_whose_children_have_no_terminal_hosts_none() {
    if has_a_terminal(std::process::id() as i32) {
        return;
    }
    let mut child = Command::new("/bin/sleep")
        .arg("5")
        .stdin(Stdio::null())
        .spawn()
        .expect("sleep starts");
    let hosts = hosts_a_terminal(std::process::id() as i32);
    let _ = child.kill();
    let _ = child.wait();
    assert!(!hosts);
}
