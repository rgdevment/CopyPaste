//! Lo que un archivo de audio o vídeo sabe decir de sí mismo.

use objc2_av_foundation::AVURLAsset;
use objc2_foundation::{NSString, NSURL};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaInfo {
    /// En segundos. Es lo que la tarjeta pinta como «3:47».
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

    /// Lo que merece entrar en el índice: quien busca «la canción de los
    /// créditos» escribe el título o el artista, nunca el nombre del archivo.
    pub fn searchable(&self) -> String {
        [&self.title, &self.artist, &self.album]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Lee lo que el archivo declara, sin decodificar su contenido.
pub fn info_for(path: &std::path::Path) -> Option<MediaInfo> {
    if !path.exists() {
        return None;
    }
    let text = NSString::from_str(path.to_str()?);
    let url = NSURL::fileURLWithPath(&text);
    // SAFETY: la URL es válida y sin opciones se usan las de por defecto.
    let asset = unsafe { AVURLAsset::URLAssetWithURL_options(&url, None) };

    let mut info = MediaInfo::default();

    // SAFETY: el asset sigue vivo; devuelve una estructura por valor.
    let duration = unsafe { asset.duration() };
    if duration.timescale != 0 {
        let seconds = duration.value as f64 / duration.timescale as f64;
        if seconds.is_finite() && seconds > 0.0 {
            info.duration = Some(seconds);
        }
    }

    // La resolución sale de la primera pista de vídeo, si la hay: un archivo
    // de audio no tiene ninguna y eso ya es la respuesta.
    // SAFETY: el asset sigue vivo mientras se recorren sus pistas.
    let tracks = unsafe { asset.tracks() };
    for track in tracks.iter() {
        // SAFETY: la pista pertenece al array, que sigue vivo.
        let size = unsafe { track.naturalSize() };
        if size.width >= 1.0 && size.height >= 1.0 {
            info.width = Some(size.width as u32);
            info.height = Some(size.height as u32);
            break;
        }
    }

    (!info.is_empty()).then_some(info)
}
