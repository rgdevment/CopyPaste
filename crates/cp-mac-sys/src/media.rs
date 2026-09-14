use objc2_av_foundation::AVURLAsset;
use objc2_foundation::{NSString, NSURL};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaInfo {
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

impl MediaInfo {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    pub fn searchable(&self) -> String {
        [&self.title, &self.artist, &self.album]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub fn info_for(path: &std::path::Path) -> Option<MediaInfo> {
    if !path.exists() {
        return None;
    }
    let text = NSString::from_str(path.to_str()?);
    let url = NSURL::fileURLWithPath(&text);

    let asset = unsafe { AVURLAsset::URLAssetWithURL_options(&url, None) };

    let mut info = MediaInfo::default();

    let duration = unsafe { asset.duration() };
    if duration.timescale != 0 {
        let seconds = duration.value as f64 / duration.timescale as f64;
        if seconds.is_finite() && seconds > 0.0 {
            info.duration = Some(seconds);
        }
    }

    let tracks = unsafe { asset.tracks() };
    for track in tracks.iter() {
        let size = unsafe { track.naturalSize() };
        if size.width >= 1.0 && size.height >= 1.0 {
            info.width = Some(size.width as u32);
            info.height = Some(size.height as u32);
            break;
        }
    }

    (!info.is_empty()).then_some(info)
}
