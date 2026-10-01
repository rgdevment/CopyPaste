use super::*;

#[test]
fn nothing_heard_is_no_wave_at_all() {
    assert!(shrunk(&[]).is_empty());
}

#[test]
fn a_wave_always_has_the_same_number_of_bars() {
    for how_many in [1usize, 2, 7, BARS, BARS + 1, 500, 3_000] {
        let peaks: Vec<f32> = (0..how_many).map(|at| (at % 10) as f32 / 10.0).collect();
        assert_eq!(shrunk(&peaks).len(), BARS, "{how_many} peaks");
    }
}

#[test]
fn the_loudest_moment_reaches_the_top_and_nothing_goes_over() {
    let peaks: Vec<f32> = (0..BARS).map(|at| at as f32 / BARS as f32).collect();
    let bars = shrunk(&peaks);
    assert_eq!(*bars.last().expect("a bar"), TALLEST);
    for bar in &bars {
        assert!(*bar <= TALLEST, "{bar} is taller than the tallest");
    }
}

#[test]
fn silence_draws_a_flat_line_instead_of_dividing_by_nothing() {
    let bars = shrunk(&[0.0, 0.0, 0.0, 0.0]);
    assert_eq!(bars.len(), BARS);
    assert!(
        bars.iter().all(|bar| *bar == 0),
        "silence is flat, not loud"
    );
}

#[test]
fn a_quiet_recording_is_lifted_so_its_shape_can_be_seen() {
    let bars = shrunk(&[0.01, 0.002, 0.01]);
    assert_eq!(
        *bars.iter().max().expect("a bar"),
        TALLEST,
        "scaled to its own loudest"
    );
}

#[test]
fn every_bar_comes_from_its_own_slice_of_the_sound() {
    let mut peaks = vec![0.0f32; BARS * 4];
    peaks[0] = 1.0;
    let bars = shrunk(&peaks);
    assert_eq!(bars[0], TALLEST, "the shout is in the first bar");
    assert!(bars[1..].iter().all(|bar| *bar == 0), "and nowhere else");
}

#[test]
fn what_is_not_audio_at_all_gives_no_wave() {
    let dir = std::env::temp_dir().join(format!("cp-wave-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("made");
    let path = dir.join("no-es-audio.wav");
    std::fs::write(&path, b"ni de lejos un riff").expect("wrote");
    assert!(bars_of(&path).is_none());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_real_wav_is_read_and_its_shape_survives() {
    let dir = std::env::temp_dir().join(format!("cp-wave-ok-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("made");
    let path = dir.join("tono.wav");
    std::fs::write(&path, tone()).expect("wrote");

    let bars = bars_of(&path).expect("a wave");
    assert_eq!(bars.len(), BARS);
    assert_eq!(*bars.iter().max().expect("a bar"), TALLEST);
    assert!(
        bars[..BARS / 3].iter().all(|bar| *bar == 0),
        "the file starts in silence and the wave says so"
    );
    std::fs::remove_dir_all(&dir).ok();
}

fn tone() -> Vec<u8> {
    const RATE: u32 = 8_000;
    const FRAMES: u32 = RATE * 2;
    let mut samples: Vec<u8> = Vec::with_capacity(FRAMES as usize * 2);
    for at in 0..FRAMES {
        let loud = if at < FRAMES / 2 { 0.0 } else { 0.8 };
        let turn = (at as f32) * 440.0 * std::f32::consts::TAU / RATE as f32;
        let value = (turn.sin() * loud * f32::from(i16::MAX)) as i16;
        samples.extend_from_slice(&value.to_le_bytes());
    }
    let data = u32::try_from(samples.len()).expect("fits");
    let mut out = Vec::with_capacity(samples.len() + 44);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    out.extend_from_slice(&samples);
    out
}
