//! El mapa de bits del portapapeles de Windows, convertido a algo que se pueda
//! guardar.
//!
//! `CF_DIB` y `CF_DIBV5` llegan sin comprimir y sin la cabecera de archivo que
//! los haría un `.bmp`. Medido el 14/09/2026 en Windows 11 26200, la misma
//! imagen ocupa **480.052 bytes en DIB y 1.964 en PNG**: guardar el DIB tal
//! cual es pagar 244 veces el precio por cada captura.
//!
//! Todo lo de aquí es de bytes a bytes, así que se prueba sin Windows delante.

const FILE_HEADER: usize = 14;
const INFO_HEADER: usize = 40;
const BI_BITFIELDS: u32 = 3;
const RGBQUAD: usize = 4;

/// Lo que dice la cabecera de un DIB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub size: u32,
    pub width: i32,
    pub height: i32,
    pub bit_count: u16,
    pub compression: u32,
    pub clr_used: u32,
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    bytes
        .get(at..at + 4)
        .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
}

pub fn header(dib: &[u8]) -> Option<Header> {
    let size = u32_at(dib, 0)?;
    // Una cabecera que dice medir menos que la mínima, o más que el buffer
    // entero, no es una cabecera: es basura con forma de imagen.
    if (size as usize) < INFO_HEADER || size as usize > dib.len() {
        return None;
    }
    Some(Header {
        size,
        width: u32_at(dib, 4)? as i32,
        height: u32_at(dib, 8)? as i32,
        bit_count: u16::from_le_bytes([*dib.get(14)?, *dib.get(15)?]),
        compression: u32_at(dib, 16)?,
        clr_used: u32_at(dib, 32)?,
    })
}

/// Dónde empiezan los píxeles, contando desde el principio del DIB.
///
/// Es el número que decide si la imagen sale bien o sale desplazada, y tiene
/// dos trampas que la 2.x documentó tras encontrarlas en campo.
pub fn pixel_offset(dib: &[u8]) -> Option<usize> {
    let head = header(dib)?;
    let mut table = 0usize;

    // Las máscaras de `BI_BITFIELDS` solo siguen a la cabecera clásica de 40
    // bytes; en `BITMAPV4HEADER` y `BITMAPV5HEADER` van dentro, y sumarlas otra
    // vez desplaza la imagen 12 bytes.
    if head.compression == BI_BITFIELDS && head.size as usize == INFO_HEADER {
        table += 3 * 4;
    }

    if head.bit_count <= 8 {
        let colors = if head.clr_used > 0 {
            head.clr_used
        } else {
            1u32 << head.bit_count
        };
        table += colors as usize * RGBQUAD;
    } else if head.clr_used != 0 {
        // Por encima de 8 bits la paleta es opcional, y los productores dejan
        // `biClrUsed` sucio a menudo: honrarlo a ciegas manda los píxeles fuera
        // del buffer. Solo se aplica si lo que dice cabe de verdad.
        let claimed = head.clr_used as usize * RGBQUAD;
        if head.size as usize + table + claimed <= dib.len() {
            table += claimed;
        }
    }

    let offset = head.size as usize + table;
    (offset <= dib.len()).then_some(offset)
}

/// Un `.bmp` completo: el DIB con su cabecera de archivo delante.
pub fn as_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    let pixels = pixel_offset(dib)?;
    let mut bmp = Vec::with_capacity(FILE_HEADER + dib.len());
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&((FILE_HEADER + dib.len()) as u32).to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes());
    bmp.extend_from_slice(&((FILE_HEADER + pixels) as u32).to_le_bytes());
    bmp.extend_from_slice(dib);
    Some(bmp)
}

/// Qué hay de verdad en el cuarto byte de cada píxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alpha {
    /// No hay cuarto byte: la imagen no es de 32 bits.
    Absent,
    /// Lo hay y no dice nada: o está todo a cero porque nadie lo escribió, o
    /// está todo opaco. En los dos casos la imagen es opaca.
    Opaque,
    /// Hay transparencia de verdad y hay que respetarla.
    Real,
}

/// La máscara de alfa que la cabecera declara, cuando la lleva dentro.
///
/// Solo `BITMAPV4HEADER` y `BITMAPV5HEADER` la tienen, en el mismo sitio. Con
/// la cabecera clásica de 40 bytes y `BI_BITFIELDS` las máscaras van detrás,
/// pero son tres y ninguna es la del alfa.
fn declared_alpha_mask(dib: &[u8], head: Header) -> Option<u32> {
    (head.size as usize >= 56)
        .then(|| u32_at(dib, 52))
        .flatten()
}

/// Por especificación, el cuarto byte de un `BI_RGB` de 32 bits es
/// **indefinido**; en la práctica los productores modernos escriben el alfa
/// ahí. Honrar un canal entero a cero daría una imagen invisible, así que se
/// mira antes de creérselo.
///
/// Pero la cabecera manda sobre la heurística. Medido el 14/09/2026: al poner
/// solo un `CF_DIB`, Windows sintetiza un `CF_DIBV5` con `biSize` 124,
/// `BI_RGB` y **`bV5AlphaMask` a cero** —dice que no hay canal alfa— sobre los
/// mismos píxeles. Mirar únicamente el cuarto byte de un sintetizado así es
/// inventarse una transparencia y grabarla en el PNG para siempre.
pub fn alpha(dib: &[u8]) -> Alpha {
    let Some(head) = header(dib) else {
        return Alpha::Absent;
    };
    if head.bit_count != 32 {
        return Alpha::Absent;
    }
    if declared_alpha_mask(dib, head) == Some(0) {
        return Alpha::Absent;
    }
    let Some(start) = pixel_offset(dib) else {
        return Alpha::Absent;
    };
    // `pixel_offset` ya garantiza que el corte cae dentro; el `unwrap` evita
    // una rama que ninguna entrada puede alcanzar.
    let pixels = dib.get(start..).unwrap_or_default();
    let mut seen_zero = false;
    let mut seen_full = false;
    let mut seen_between = false;
    for chunk in pixels.as_chunks::<4>().0 {
        match chunk[3] {
            0x00 => seen_zero = true,
            0xFF => seen_full = true,
            _ => seen_between = true,
        }
    }
    if seen_between || (seen_zero && seen_full) {
        Alpha::Real
    } else {
        Alpha::Opaque
    }
}

/// El DIB como PNG, que es como se guarda.
///
/// Devuelve `None` si los bytes no son un mapa de bits que se pueda leer.
pub fn to_png(dib: &[u8]) -> Option<Vec<u8>> {
    let mut owned;
    // Todo lo que no sea transparencia declarada se escribe opaco: un cuarto
    // byte que nadie llenó, leído como alfa, da una imagen invisible.
    let source = if alpha(dib) != Alpha::Real && header(dib)?.bit_count == 32 {
        owned = dib.to_vec();
        let start = pixel_offset(&owned)?;
        for chunk in owned.get_mut(start..)?.as_chunks_mut::<4>().0 {
            chunk[3] = 0xFF;
        }
        &owned
    } else {
        dib
    };
    let bmp = as_bmp(source)?;
    let decoded = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp).ok()?;
    let mut out = std::io::Cursor::new(Vec::new());
    decoded
        .write_to(&mut out, image::ImageFormat::Png)
        .ok()
        .map(|()| out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un DIB armado a mano, para poder mover una pieza cada vez.
    struct Dib {
        header_size: u32,
        width: i32,
        height: i32,
        bit_count: u16,
        compression: u32,
        clr_used: u32,
        table: Vec<u8>,
        pixels: Vec<u8>,
    }

    impl Dib {
        fn rgb32(width: i32, height: i32) -> Self {
            Self {
                header_size: INFO_HEADER as u32,
                width,
                height,
                bit_count: 32,
                compression: 0,
                clr_used: 0,
                table: Vec::new(),
                pixels: [0x40, 0x80, 0xC0, 0xFF].repeat((width * height.abs()) as usize),
            }
        }

        fn build(&self) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(&self.header_size.to_le_bytes());
            out.extend_from_slice(&self.width.to_le_bytes());
            out.extend_from_slice(&self.height.to_le_bytes());
            out.extend_from_slice(&1u16.to_le_bytes());
            out.extend_from_slice(&self.bit_count.to_le_bytes());
            out.extend_from_slice(&self.compression.to_le_bytes());
            out.extend_from_slice(&(self.pixels.len() as u32).to_le_bytes());
            out.extend_from_slice(&2835i32.to_le_bytes());
            out.extend_from_slice(&2835i32.to_le_bytes());
            out.extend_from_slice(&self.clr_used.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.resize(self.header_size as usize, 0);
            out.extend_from_slice(&self.table);
            out.extend_from_slice(&self.pixels);
            out
        }
    }

    #[test]
    fn a_plain_dib_puts_the_pixels_right_after_the_header() {
        let dib = Dib::rgb32(2, 2).build();
        assert_eq!(pixel_offset(&dib), Some(INFO_HEADER));
    }

    /// La primera trampa: con la cabecera clásica, las máscaras van detrás.
    #[test]
    fn bitfield_masks_follow_the_classic_header() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.table = vec![0; 12];
        assert_eq!(pixel_offset(&dib.build()), Some(INFO_HEADER + 12));
    }

    /// Y la otra mitad de la trampa: con `BITMAPV4HEADER` y `BITMAPV5HEADER`
    /// las máscaras están dentro, y sumarlas otra vez corre la imagen 12 bytes.
    #[test]
    fn bitfield_masks_live_inside_the_newer_headers() {
        for size in [108u32, 124] {
            let mut dib = Dib::rgb32(2, 2);
            dib.header_size = size;
            dib.compression = BI_BITFIELDS;
            assert_eq!(
                pixel_offset(&dib.build()),
                Some(size as usize),
                "cabecera de {size} bytes"
            );
        }
    }

    #[test]
    fn a_palette_below_nine_bits_shifts_the_pixels() {
        let mut dib = Dib::rgb32(4, 1);
        dib.bit_count = 8;
        dib.pixels = vec![0, 1, 2, 3];
        dib.table = vec![0; 256 * RGBQUAD];
        assert_eq!(
            pixel_offset(&dib.build()),
            Some(INFO_HEADER + 256 * RGBQUAD),
            "sin biClrUsed se asume la paleta entera"
        );
    }

    #[test]
    fn a_declared_palette_size_is_honoured() {
        let mut dib = Dib::rgb32(4, 1);
        dib.bit_count = 8;
        dib.clr_used = 16;
        dib.pixels = vec![0, 1, 2, 3];
        dib.table = vec![0; 16 * RGBQUAD];
        assert_eq!(pixel_offset(&dib.build()), Some(INFO_HEADER + 16 * RGBQUAD));
    }

    /// La segunda trampa: por encima de 8 bits los productores dejan
    /// `biClrUsed` sucio, y creérselo manda los píxeles fuera del buffer.
    #[test]
    fn a_dirty_palette_count_above_eight_bits_is_ignored() {
        let mut dib = Dib::rgb32(2, 2);
        dib.clr_used = 1_000_000;
        assert_eq!(
            pixel_offset(&dib.build()),
            Some(INFO_HEADER),
            "lo que no cabe no se resta"
        );
    }

    #[test]
    fn a_palette_that_does_fit_above_eight_bits_is_applied() {
        let mut dib = Dib::rgb32(2, 2);
        dib.clr_used = 2;
        dib.table = vec![0; 2 * RGBQUAD];
        assert_eq!(pixel_offset(&dib.build()), Some(INFO_HEADER + 2 * RGBQUAD));
    }

    /// Una cabecera que ocupa el buffer entero sigue siendo legible: dice lo
    /// que dice, y que detrás no venga un solo píxel es otro asunto.
    #[test]
    fn a_header_that_fills_the_whole_buffer_is_still_a_header() {
        let mut dib = Dib::rgb32(1, 1);
        dib.pixels = Vec::new();
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER);
        assert_eq!(
            header(&raw).map(|head| head.size),
            Some(INFO_HEADER as u32),
            "medir justo lo que hay no es medir de más"
        );
        assert_eq!(pixel_offset(&raw), Some(INFO_HEADER));
    }

    /// La comprobación de que la paleta cabe suma los tres tramos: cabecera,
    /// máscaras y paleta. Con la paleta justo fuera del buffer, la cuenta tiene
    /// que dar que no cabe y quedarse con lo anterior.
    #[test]
    fn a_palette_one_byte_too_long_is_left_out() {
        let mut dib = Dib::rgb32(2, 2);
        dib.clr_used = 2;
        dib.pixels = vec![0; 5];
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER + 5);
        assert_eq!(
            pixel_offset(&raw),
            Some(INFO_HEADER),
            "ocho bytes de paleta no caben en cinco"
        );
    }

    /// Y lo mismo con las máscaras por delante: los tres tramos se suman, no
    /// se restan ni se multiplican entre sí.
    #[test]
    fn the_masks_count_towards_whether_the_palette_fits() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.clr_used = 3;
        dib.table = vec![0; 12];
        dib.pixels = vec![0; 8];
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER + 12 + 8);
        assert_eq!(
            pixel_offset(&raw),
            Some(INFO_HEADER + 12),
            "doce de máscaras más doce de paleta no caben en veinte"
        );
    }

    #[test]
    fn nonsense_is_not_a_bitmap() {
        assert_eq!(header(&[]), None);
        assert_eq!(header(&[0; 8]), None, "ni siquiera llega a la cabecera");
        assert_eq!(header(&[0; 40]), None, "una cabecera que dice medir cero");
        let mut lying = Dib::rgb32(2, 2).build();
        lying[0..4].copy_from_slice(&9999u32.to_le_bytes());
        assert_eq!(header(&lying), None, "dice medir más que todo el buffer");
        assert_eq!(as_bmp(&[]), None);
        assert_eq!(to_png(&[]), None);
    }

    #[test]
    fn the_file_header_points_at_the_pixels() {
        let dib = Dib::rgb32(2, 2).build();
        let bmp = as_bmp(&dib).expect("bmp");
        assert_eq!(&bmp[0..2], b"BM");
        assert_eq!(
            u32::from_le_bytes([bmp[2], bmp[3], bmp[4], bmp[5]]) as usize,
            FILE_HEADER + dib.len(),
            "el tamaño declarado es el del archivo entero"
        );
        assert_eq!(
            u32::from_le_bytes([bmp[10], bmp[11], bmp[12], bmp[13]]) as usize,
            FILE_HEADER + INFO_HEADER
        );
        assert_eq!(&bmp[FILE_HEADER..], &dib[..], "el DIB viaja intacto");
    }

    #[test]
    fn an_alpha_channel_nobody_wrote_is_not_transparency() {
        let mut dib = Dib::rgb32(2, 2);
        dib.pixels = [0x40, 0x80, 0xC0, 0x00].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Opaque);
    }

    #[test]
    fn an_alpha_channel_fully_opaque_is_opaque() {
        let dib = Dib::rgb32(2, 2).build();
        assert_eq!(alpha(&dib), Alpha::Opaque);
    }

    #[test]
    fn a_mixed_alpha_channel_is_real_transparency() {
        let mut dib = Dib::rgb32(2, 2);
        dib.pixels = vec![
            0x40, 0x80, 0xC0, 0xFF, //
            0x40, 0x80, 0xC0, 0x00, //
            0x40, 0x80, 0xC0, 0xFF, //
            0x40, 0x80, 0xC0, 0xFF,
        ];
        assert_eq!(
            alpha(&dib.build()),
            Alpha::Real,
            "unos a cero y otros opacos es un recorte con bordes"
        );
    }

    #[test]
    fn a_partial_alpha_value_is_real_transparency() {
        let mut dib = Dib::rgb32(2, 2);
        dib.pixels = [0x40, 0x80, 0xC0, 0x7F].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Real);
    }

    /// Una cabecera que promete máscaras que el buffer no tiene: no hay
    /// píxeles donde mirar, así que no hay alfa que leer.
    #[test]
    fn a_header_promising_more_than_the_buffer_holds_has_no_alpha() {
        let mut dib = Dib::rgb32(1, 1);
        dib.compression = BI_BITFIELDS;
        dib.pixels = Vec::new();
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER);
        assert_eq!(pixel_offset(&raw), None, "las máscaras no caben");
        assert_eq!(alpha(&raw), Alpha::Absent);
        assert_eq!(to_png(&raw), None);
    }

    /// El caso medido el 14/09/2026: al poner solo un `CF_DIB`, Windows
    /// sintetiza un `CF_DIBV5` de 124 bytes de cabecera, `BI_RGB`, con la
    /// máscara de alfa a cero. La cabecera manda: no hay canal que leer.
    #[test]
    fn a_synthesised_v5_declaring_no_alpha_mask_has_no_alpha() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 124;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        let raw = dib.build();
        assert_eq!(u32::from_le_bytes([raw[52], raw[53], raw[54], raw[55]]), 0);
        assert_eq!(
            alpha(&raw),
            Alpha::Absent,
            "el cuarto byte parece alfa, pero la cabecera dice que no lo es"
        );
    }

    /// Y sin esa corrección la imagen saldría medio transparente: el PNG tiene
    /// que quedar opaco.
    #[test]
    fn a_synthesised_v5_does_not_become_half_transparent() {
        let mut dib = Dib::rgb32(4, 4);
        dib.header_size = 124;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(16);
        let png = to_png(&dib.build()).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        assert!(
            back.pixels().all(|pixel| pixel[3] == 0xFF),
            "una transparencia que nadie declaró no se graba"
        );
    }

    /// La cabecera más corta que llega a declarar el alfa mide 56 bytes: es la
    /// `BITMAPV3INFOHEADER` que escriben Photoshop y GIMP, cuatro más que la de
    /// tres máscaras. Pedir 57 la dejaría fuera y su transparencia se perdería.
    #[test]
    fn the_shortest_header_that_declares_alpha_is_fifty_six_bytes() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 56;
        dib.compression = BI_BITFIELDS;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        let mut raw = dib.build();
        raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
        raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(pixel_offset(&raw), Some(56), "las máscaras van dentro");
        assert_eq!(
            alpha(&raw),
            Alpha::Real,
            "declara la máscara de alfa y hay que leerla"
        );
    }

    /// Y la misma cabecera de 56 con la máscara a cero no tiene alfa, igual
    /// que la de 124: lo que decide es que el campo esté, no cuánto mide.
    #[test]
    fn a_fifty_six_byte_header_with_no_mask_has_no_alpha() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 56;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Absent);
    }

    /// La máscara declarada distingue el sintetizado del real: con una máscara
    /// de verdad, los mismos píxeles sí llevan alfa.
    #[test]
    fn a_declared_mask_turns_the_same_pixels_into_real_alpha() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 124;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        let mut raw = dib.build();
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(alpha(&raw), Alpha::Real);
    }

    /// La cabecera clásica no lleva máscara de alfa ni cuando usa
    /// `BI_BITFIELDS`: allí solo hay tres, y la heurística sigue mandando.
    #[test]
    fn the_classic_header_has_no_alpha_mask_to_declare() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.table = vec![0; 12];
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Real);
    }

    #[test]
    fn below_thirty_two_bits_there_is_no_alpha_to_read() {
        let mut dib = Dib::rgb32(2, 2);
        dib.bit_count = 24;
        dib.pixels = [0x40, 0x80, 0xC0].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Absent);
    }

    /// El número que justifica todo este módulo.
    #[test]
    fn a_bitmap_becomes_a_fraction_of_its_size_as_png() {
        let mut dib = Dib::rgb32(200, 200);
        dib.pixels = [0x20, 0x60, 0xA0, 0xFF].repeat(200 * 200);
        let raw = dib.build();
        let png = to_png(&raw).expect("png");
        assert!(
            png.len() * 20 < raw.len(),
            "{} bytes de DIB contra {} de PNG",
            raw.len(),
            png.len()
        );
    }

    #[test]
    fn a_png_from_a_dib_is_a_png() {
        let dib = Dib::rgb32(4, 4).build();
        let png = to_png(&dib).expect("png");
        assert_eq!(&png[1..4], b"PNG");
        let back = image::load_from_memory(&png).expect("se relee");
        assert_eq!((back.width(), back.height()), (4, 4));
    }

    /// Un canal alfa a cero se corrige **antes** de decodificar: si no, el PNG
    /// sale entero transparente y la captura se pierde sin que nadie lo vea.
    #[test]
    fn a_capture_with_an_unwritten_alpha_does_not_become_invisible() {
        let mut dib = Dib::rgb32(4, 4);
        dib.pixels = [0x20, 0x60, 0xA0, 0x00].repeat(16);
        let png = to_png(&dib.build()).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        assert!(
            back.pixels().all(|pixel| pixel[3] == 0xFF),
            "la imagen tiene que verse"
        );
    }

    /// El caso que de verdad salva la imagen: la cabecera **declara** la
    /// máscara de alfa, así que el decodificador va a leer ese canal, y la
    /// fuente lo dejó entero a cero. Sin corregirlo, la captura se guarda
    /// completamente transparente y el usuario ve una tarjeta vacía.
    #[test]
    fn a_declared_alpha_channel_left_at_zero_is_not_an_invisible_capture() {
        let mut dib = Dib::rgb32(4, 4);
        dib.header_size = 124;
        dib.compression = BI_BITFIELDS;
        dib.pixels = [0x20, 0x60, 0xA0, 0x00].repeat(16);
        let mut raw = dib.build();
        raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
        raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(
            alpha(&raw),
            Alpha::Opaque,
            "todo a cero no es transparencia"
        );
        let png = to_png(&raw).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        assert!(
            back.pixels().all(|pixel| pixel[3] == 0xFF),
            "la captura tiene que verse"
        );
    }

    /// Y la transparencia de verdad sobrevive al viaje, que es la otra mitad:
    /// forzar el alfa siempre es el fallo que la 2.x tiene al escribir.
    #[test]
    fn real_transparency_survives_the_trip() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.header_size = 124;
        dib.pixels = vec![
            0x20, 0x60, 0xA0, 0x00, //
            0x20, 0x60, 0xA0, 0x80, //
            0x20, 0x60, 0xA0, 0xC0, //
            0x20, 0x60, 0xA0, 0xFF,
        ];
        let mut raw = dib.build();
        // Las máscaras de un BITMAPV5HEADER, en su sitio dentro de la cabecera.
        raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
        raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(alpha(&raw), Alpha::Real);
        let png = to_png(&raw).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        let seen: Vec<u8> = back.pixels().map(|pixel| pixel[3]).collect();
        assert!(
            seen.contains(&0x00) && seen.contains(&0xFF),
            "los cuatro niveles de alfa llegaron como {seen:?}"
        );
    }

    /// Un DIB cuyos píxeles no llegan hasta donde la cabecera promete no puede
    /// tumbar el proceso: se dice que no en vez de indexar fuera.
    #[test]
    fn a_truncated_bitmap_is_refused_not_panicked_on() {
        let dib = Dib::rgb32(100, 100).build();
        for cut in [0, 1, 20, 39, 40, 41, 100, 500] {
            let short = &dib[..cut.min(dib.len())];
            let _ = header(short);
            let _ = pixel_offset(short);
            let _ = alpha(short);
            let _ = as_bmp(short);
            let _ = to_png(short);
        }
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Ningún montón de bytes puede tumbar el proceso. El portapapeles lo
        /// llena cualquiera, así que esto no es una precaución teórica.
        #[test]
        fn no_pile_of_bytes_can_bring_the_process_down(
            bytes in prop::collection::vec(any::<u8>(), 0..600),
        ) {
            let _ = header(&bytes);
            let _ = pixel_offset(&bytes);
            let _ = alpha(&bytes);
            let _ = as_bmp(&bytes);
            let _ = to_png(&bytes);
        }

        /// Donde empiezan los píxeles cae siempre dentro del buffer: es el
        /// número con el que después se indexa.
        #[test]
        fn the_pixels_always_start_inside_the_buffer(
            bytes in prop::collection::vec(any::<u8>(), 0..600),
        ) {
            if let Some(at) = pixel_offset(&bytes) {
                prop_assert!(at <= bytes.len());
                prop_assert!(at >= super::INFO_HEADER);
            }
        }

        /// El `.bmp` es el DIB con catorce bytes delante, ni uno más.
        #[test]
        fn the_bmp_is_the_dib_with_a_header_in_front(
            bytes in prop::collection::vec(any::<u8>(), 0..600),
        ) {
            if let Some(bmp) = as_bmp(&bytes) {
                prop_assert_eq!(bmp.len(), bytes.len() + super::FILE_HEADER);
                prop_assert_eq!(&bmp[super::FILE_HEADER..], &bytes[..]);
            }
        }
    }
}
