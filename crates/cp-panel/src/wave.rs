use std::path::Path;
use symphonia::core::audio::AudioBufferRef;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub const BARS: usize = 48;
pub const LISTEN_FOR: std::time::Duration = std::time::Duration::from_secs(30);
pub const TALLEST: u8 = 100;

const _: () = assert!(BARS >= 8);
const _: () = assert!(LISTEN_FOR.as_secs() >= 1);

pub fn bars_of(path: &Path) -> Option<Vec<u8>> {
    let file = std::fs::File::open(path).ok()?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(after) = path.extension().and_then(|one| one.to_str()) {
        hint.with_extension(after);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;
    let mut reader = probed.format;
    let track = reader
        .tracks()
        .iter()
        .find(|one| one.codec_params.channels.is_some())?;
    let wanted = track.id;
    let rate = track.codec_params.sample_rate.unwrap_or(44_100);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .ok()?;

    let ceiling = u64::from(rate) * LISTEN_FOR.as_secs();
    let mut heard = 0u64;
    let mut peaks: Vec<f32> = Vec::new();
    while heard < ceiling {
        let packet = match reader.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };
        if packet.track_id() != wanted {
            continue;
        }
        let Ok(decoded) = decoder.decode(&packet) else {
            continue;
        };
        heard += decoded.frames() as u64;
        peaks.push(loudest(&decoded));
    }
    (!peaks.is_empty()).then(|| shrunk(&peaks))
}

fn loudest(decoded: &AudioBufferRef<'_>) -> f32 {
    use symphonia::core::audio::Signal;
    match decoded {
        AudioBufferRef::F32(buffer) => buffer
            .chan(0)
            .iter()
            .fold(0.0f32, |most, one| most.max(one.abs())),
        AudioBufferRef::S32(buffer) => {
            let scale = i32::MAX as f32;
            buffer
                .chan(0)
                .iter()
                .fold(0.0f32, |most, one| most.max((*one as f32 / scale).abs()))
        }
        AudioBufferRef::S16(buffer) => {
            let scale = i16::MAX as f32;
            buffer
                .chan(0)
                .iter()
                .fold(0.0f32, |most, one| most.max((*one as f32 / scale).abs()))
        }
        AudioBufferRef::U8(buffer) => buffer.chan(0).iter().fold(0.0f32, |most, one| {
            most.max(((f32::from(*one) - 128.0) / 128.0).abs())
        }),
        _ => 0.0,
    }
}

pub fn shrunk(peaks: &[f32]) -> Vec<u8> {
    if peaks.is_empty() {
        return Vec::new();
    }
    let tallest = peaks.iter().fold(0.0f32, |most, one| most.max(*one));
    let scale = if tallest > 0.0 { tallest } else { 1.0 };
    (0..BARS)
        .map(|bar| {
            let from = bar * peaks.len() / BARS;
            let to = ((bar + 1) * peaks.len() / BARS)
                .max(from + 1)
                .min(peaks.len());
            let slice = &peaks[from..to];
            let loudest = slice.iter().fold(0.0f32, |most, one| most.max(*one));
            let height = (loudest / scale * f32::from(TALLEST)).round();
            height.clamp(0.0, f32::from(TALLEST)) as u8
        })
        .collect()
}

#[cfg(test)]
#[path = "wave_test.rs"]
mod tests;
