use std::path::Path;
use symphonia::core::audio::{Audio, GenericAudioBufferRef};
use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

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
    let mut reader = symphonia::default::get_probe()
        .probe(
            &hint,
            stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .ok()?;
    let (wanted, params) = reader
        .tracks()
        .iter()
        .find_map(|one| match one.codec_params {
            Some(CodecParameters::Audio(ref audio)) if audio.channels.is_some() => {
                Some((one.id, audio.clone()))
            }
            _ => None,
        })?;
    let rate = params.sample_rate.unwrap_or(44_100);
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&params, &AudioDecoderOptions::default())
        .ok()?;

    let ceiling = u64::from(rate) * LISTEN_FOR.as_secs();
    let mut heard = 0u64;
    let mut peaks: Vec<f32> = Vec::new();
    while heard < ceiling {
        let Ok(Some(packet)) = reader.next_packet() else {
            break;
        };
        if packet.track_id != wanted {
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

fn peak_of<S: Copy>(plane: Option<&[S]>, into: impl Fn(S) -> f32) -> f32 {
    plane
        .map(|one| one.iter().fold(0.0f32, |most, s| most.max(into(*s).abs())))
        .unwrap_or(0.0)
}

fn loudest(decoded: &GenericAudioBufferRef<'_>) -> f32 {
    match decoded {
        GenericAudioBufferRef::F32(one) => peak_of(one.plane(0), |s| s),
        GenericAudioBufferRef::F64(one) => peak_of(one.plane(0), |s| s as f32),
        GenericAudioBufferRef::S32(one) => peak_of(one.plane(0), |s| s as f32 / i32::MAX as f32),
        GenericAudioBufferRef::S16(one) => {
            peak_of(one.plane(0), |s| f32::from(s) / f32::from(i16::MAX))
        }
        GenericAudioBufferRef::S8(one) => {
            peak_of(one.plane(0), |s| f32::from(s) / f32::from(i8::MAX))
        }
        GenericAudioBufferRef::U8(one) => peak_of(one.plane(0), |s| (f32::from(s) - 128.0) / 128.0),
        GenericAudioBufferRef::U16(one) => {
            peak_of(one.plane(0), |s| (f32::from(s) - 32_768.0) / 32_768.0)
        }
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
