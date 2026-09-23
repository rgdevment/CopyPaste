use cp_config::Config;
use std::time::Instant;

const TURNS: usize = 200;

fn main() {
    let room = tempfile::tempdir().expect("una carpeta temporal");
    let path = cp_config::at(room.path());
    let mut kept = Config::default();

    let mut writing = Vec::with_capacity(TURNS);
    let mut reading = Vec::with_capacity(TURNS);

    for turn in 0..TURNS {
        kept.keeps_days = Some((turn % 90) as u16 + 1);

        let at = Instant::now();
        cp_config::write(&path, &kept).expect("escribe");
        writing.push(at.elapsed().as_micros());

        let at = Instant::now();
        let back = cp_config::read(&path).expect("lee");
        reading.push(at.elapsed().as_micros());

        assert_eq!(back.keeps_days, kept.keeps_days);
    }

    let writing_slow = tell("escribir", &mut writing);
    let reading_slow = tell("leer", &mut reading);
    if writing_slow || reading_slow {
        std::process::exit(1);
    }
}

const CEILING: u128 = 20_000;

fn tell(what: &str, times: &mut [u128]) -> bool {
    times.sort_unstable();
    let p50 = times[times.len() / 2];
    let p95 = times[times.len() * 95 / 100];
    let worst = times[times.len() - 1];
    println!("{what}: p50 {p50} µs · p95 {p95} µs · peor {worst} µs");
    if p95 > CEILING {
        println!("::error::{what} pasó de {CEILING} µs en p95: {p95} µs");
        return true;
    }
    false
}
