use crate::Panel;
use crate::app::App;
use slint::{ComponentHandle, Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

const TYPED: &str = "reunión jue";
const KEYSTROKE: Duration = Duration::from_millis(100);
const TICK: Duration = Duration::from_millis(16);
const SCROLL_STEP: f32 = 60.0;
const SCROLL_FOR: Duration = Duration::from_secs(6);
const ROW_PITCH: f32 = 182.0;

type Samples = Rc<RefCell<Vec<f64>>>;

pub fn run(app: &App) {
    let app = app.clone();
    let ui = app.ui();
    let renders: Samples = Rc::new(RefCell::new(Vec::new()));
    if let Some(panel) = ui.upgrade() {
        let renders = renders.clone();
        let began = Rc::new(RefCell::new(None::<Instant>));
        let hooked = panel
            .window()
            .set_rendering_notifier(move |state, _| match state {
                slint::RenderingState::BeforeRendering => {
                    *began.borrow_mut() = Some(Instant::now())
                }
                slint::RenderingState::AfterRendering => {
                    if let Some(started) = began.borrow_mut().take() {
                        renders
                            .borrow_mut()
                            .push(started.elapsed().as_secs_f64() * 1000.0);
                    }
                }
                _ => {}
            });
        if let Err(why) = hooked {
            eprintln!("sin notificador de render: {why:?}");
        }
    }
    let searches: Samples = Rc::new(RefCell::new(Vec::new()));
    let typed = Rc::new(RefCell::new(0usize));
    let timer = Rc::new(Timer::default());
    let keystrokes = {
        let app = app.clone();
        let timer = timer.clone();
        move || {
            let mut at = typed.borrow_mut();
            *at += 1;
            let text: String = TYPED.chars().take(*at).collect();
            if let Some(ui) = ui.upgrade() {
                ui.set_query(text.clone().into());
            }
            app.search(&text);
            searches
                .borrow_mut()
                .push(app.last_refresh().as_secs_f64() * 1000.0);
            if *at >= TYPED.chars().count() {
                timer.stop();
                let samples = searches.borrow().clone();
                report("search", "typing", samples.len(), 0, &samples);
                app.search("");
                if let Some(ui) = ui.upgrade() {
                    ui.set_query("".into());
                }
                let second = app.clone();
                let renders_images = renders.clone();
                let renders_text = renders.clone();
                renders.borrow_mut().clear();
                app.choose_chip("image");
                scroll(ui.clone(), "image", move |ui| {
                    let frames = renders_images.borrow().clone();
                    report("render", "image", frames.len(), 0, &frames);
                    renders_images.borrow_mut().clear();
                    second.choose_chip("text");
                    scroll(ui, "text", move |_| {
                        let frames = renders_text.borrow().clone();
                        report("render", "text", frames.len(), 0, &frames);
                        let _ = slint::quit_event_loop();
                    });
                });
            }
        }
    };
    timer.start(TimerMode::Repeated, KEYSTROKE, keystrokes);
    std::mem::forget(timer);
}

fn scroll(
    ui: slint::Weak<Panel>,
    over: &'static str,
    then: impl FnOnce(slint::Weak<Panel>) + 'static,
) {
    let timer = Rc::new(Timer::default());
    let intervals: Samples = Rc::new(RefCell::new(Vec::new()));
    let then = Rc::new(RefCell::new(Some(then)));
    let started = Instant::now();
    let last = Rc::new(RefCell::new(Instant::now()));
    let ticker = {
        let timer = timer.clone();
        move || {
            let now = Instant::now();
            let gap = now.duration_since(*last.borrow()).as_secs_f64() * 1000.0;
            *last.borrow_mut() = now;
            intervals.borrow_mut().push(gap);
            if let Some(ui) = ui.upgrade() {
                ui.set_scroll_y(ui.get_scroll_y() - SCROLL_STEP);
            }
            if started.elapsed() >= SCROLL_FOR {
                timer.stop();
                let samples: Vec<f64> = intervals.borrow().iter().skip(1).copied().collect();
                let rows = ui
                    .upgrade()
                    .map(|ui| (-ui.get_scroll_y() / ROW_PITCH) as i64)
                    .unwrap_or(0);
                report("scroll", over, samples.len(), rows, &samples);
                if let Some(then) = then.borrow_mut().take() {
                    then(ui.clone());
                }
            }
        }
    };
    timer.start(TimerMode::Repeated, TICK, ticker);
    std::mem::forget(timer);
}

fn report(gate: &str, over: &str, samples: usize, rows: i64, values: &[f64]) {
    println!(
        "{{\"gate\":\"{gate}\",\"over\":\"{over}\",\"samples\":{samples},\"rows\":{rows},\"p50_ms\":{:.1},\"p95_ms\":{:.1},\"max_ms\":{:.1}}}",
        percentile(values, 50.0),
        percentile(values, 95.0),
        percentile(values, 100.0)
    );
}

fn percentile(values: &[f64], pct: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("números"));
    let rank = ((pct / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_percentile_is_the_nearest_rank_and_survives_an_empty_sample() {
        let sample = [5.0, 1.0, 3.0, 2.0, 4.0];
        assert_eq!(percentile(&sample, 50.0), 3.0);
        assert_eq!(percentile(&sample, 95.0), 5.0);
        assert_eq!(percentile(&sample, 0.0), 1.0);
        assert_eq!(percentile(&[], 95.0), 0.0);
    }
}
