use crate::Panel;
use crate::app::{dress, forward, handle_of};
use crate::aside::aimed_at;
use crate::here;
use crate::note::note;
use crate::showing::{SLOW, appear, place, vanish};
use slint::ComponentHandle;
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::{Duration, Instant};

const ORDER_UP_TO: u64 = 64;

pub fn listen(
    ui: slint::Weak<Panel>,
    ahead: Arc<AtomicIsize>,
    backdrop: String,
    shelf: crate::kept::Shelf,
) {
    std::thread::spawn(move || {
        use std::io::{BufRead, Read};
        let host = here::host();
        let input = std::io::stdin();
        let mut reader = std::io::BufReader::new(input.lock());
        let mut said = String::new();
        loop {
            said.clear();
            match (&mut reader).take(ORDER_UP_TO).read_line(&mut said) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            match said.trim() {
                "show" => {
                    let asked = Instant::now();
                    let (in_front, aim) = (here::ahead_now(host), ahead.clone());
                    let kept = shelf.renew();
                    crate::say::adopt_english(kept.english);
                    let dressed = backdrop.clone();
                    let _ = ui.upgrade_in_event_loop(move |panel| {
                        let before = aim.load(Ordering::Relaxed);
                        let showing = panel.window().is_visible();
                        aim.store(aimed_at(in_front, before, showing), Ordering::Relaxed);
                        place(&panel);
                        crate::view::dress_words(&panel);
                        panel.invoke_fresh_start();
                        if panel.show().is_err() {
                            return;
                        }
                        place(&panel);
                        dress(&panel, &dressed, kept.light(here::system_is_light()));
                        forward(&panel);
                        appear(&panel);
                        panel.invoke_focus_search();
                        let took = asked.elapsed();
                        if took > SLOW {
                            note(&format!("the panel took {took:?} to show"));
                        }
                    });
                }
                "empty" => {
                    let _ = ui.upgrade_in_event_loop(|panel| panel.invoke_emptied());
                }
                "hide" => {
                    ahead.store(0, Ordering::Relaxed);
                    let _ = ui.upgrade_in_event_loop(|panel| vanish(&panel));
                }
                "quit" => break,
                _ => note("an order arrived that means nothing here"),
            }
        }
        let _ = ui.upgrade_in_event_loop(|panel| {
            if let Some(handle) = handle_of(&panel) {
                here::ground(handle);
            }
            let _ = slint::quit_event_loop();
        });
    });
}

pub fn watch_signals(ui: slint::Weak<Panel>, dir: std::path::PathBuf, shelf: crate::kept::Shelf) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(40));
            for (name, show) in [("show", true), ("hide", false)] {
                let flag = dir.join(name);
                if flag.exists() {
                    let _ = std::fs::remove_file(&flag);
                    if show {
                        crate::say::adopt_english(shelf.renew().english);
                    }
                    let _ = ui.upgrade_in_event_loop(move |ui| {
                        if show {
                            crate::view::dress_words(&ui);
                            ui.invoke_fresh_start();
                            let _ = ui.show();
                            appear(&ui);
                            ui.invoke_focus_search();
                        } else {
                            vanish(&ui);
                        }
                    });
                }
            }
        }
    });
}
