use cp_core::item::{Format, Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_core::kind::{self, Kind};
use cp_store::Store;
use std::path::PathBuf;
use std::time::Instant;

const ITEMS: usize = 50_000;
const THUMB_SIDE: u32 = 256;
const DAY: i64 = 86_400_000;

const APPS: [&str; 9] = [
    "Safari",
    "Mail",
    "Vivaldi",
    "Microsoft Teams",
    "Excel",
    "Word",
    "Recortes",
    "Orca",
    "Explorador",
];

const SENTENCES: [&str; 12] = [
    "Confirmamos la reunión del jueves a las 16:30 en la sala Ventura.",
    "El orden del día es el cierre del núcleo de macOS y la batería de pruebas de sistema.",
    "Si alguien no puede, que avise antes del miércoles.",
    "The invoice for September is attached; payment is due within thirty days.",
    "東京の天気は晴れ、明日は雨の予報です。",
    "مرحبا بالعالم، هذا نص تجريبي للبحث.",
    "Straße, encyclopædia y otras palabras con caracteres fuera del ASCII.",
    "🚀 Lanzamos la 3.1 con emoji a color y búsqueda instantánea 🎉",
    "Recuerda renovar el certificado antes del 30 de octubre.",
    "La cuota de blobs se barre a las 03:00 con el equipo en reposo.",
    "Pedido AB-4417: entrega el 12 de marzo, dos bultos, sin firma.",
    "Nada de lo que se copia desde Excel se guarda: la fuente pide no registrarse.",
];

const SPECIAL: [&str; 8] = [
    "{\"id\":\"zk5whptc-0033\",\"items\":412,\"ocr\":true,\"since\":\"2026-09-11\"}",
    "fn main() {\n    println!(\"hola\");\n}",
    "https://ejemplo.test/ruta/larga?con=parametros&y=mas",
    "correo@ejemplo.test",
    "#FF8800",
    "192.168.10.1",
    "7ab3f6de-1c4b-4f5e-8a2d-9f0e1b2c3d4e",
    "+34 600 123 456",
];

fn main() {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("cp-seed"));
    let count: usize = std::env::args()
        .nth(2)
        .and_then(|v| v.parse().ok())
        .unwrap_or(ITEMS);
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("limpiar");
    }
    std::fs::create_dir_all(root.join("thumbs")).expect("carpeta");
    let store = Store::open(&root.join("history.db")).expect("abrir");
    store.without_autocheckpoint().expect("wal");
    let started = Instant::now();
    let now = 1_789_990_000_000i64;
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut images = 0;
    let mut files = 0;
    for at in 0..count {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let roll = (seed >> 33) % 100;
        let created = now - (at as i64) * (DAY / 400);
        let uuid = format!("seed-{at:06}");
        let id = if roll < 70 {
            let text = text_at(at, seed);
            let item = text_item(&text);
            store
                .insert_item(&uuid, &item, &text, created)
                .expect("texto")
        } else if roll < 90 {
            images += 1;
            let png = thumbnail_png(at, seed);
            let path = root.join("thumbs").join(format!("{at:06}.png"));
            std::fs::write(&path, &png).expect("miniatura");
            let item = Item {
                kind: Some(Kind::Image),
                formats: vec![Format {
                    id: SYNTHETIC_IMAGE.into(),
                    payload: Payload::stored(png),
                }],
            };
            let id = store
                .insert_item(&uuid, &item, "", created)
                .expect("imagen");
            store
                .set_thumb(id, Some(&path.to_string_lossy()), created)
                .expect("thumb");
            store
                .set_ocr_text(
                    id,
                    &format!("Pedido AB-{} entrega {}", 4000 + at % 900, at % 28 + 1),
                    created,
                )
                .expect("ocr");
            id
        } else {
            files += 1;
            let directory = at.is_multiple_of(9);
            let path = if directory {
                folder_path_at(at, seed)
            } else {
                file_path_at(at, seed)
            };
            let item = Item {
                kind: Some(kind::classify_file(&path, directory)),
                formats: vec![Format {
                    id: "public.file-url".into(),
                    payload: Payload::stored(format!("file://{path}").into_bytes()),
                }],
            };
            store
                .insert_item(&uuid, &item, &path, created)
                .expect("archivo")
        };
        store
            .set_source(id, APPS[(seed >> 40) as usize % APPS.len()], created)
            .expect("origen");
        if at.is_multiple_of(97) {
            store.set_pinned(id, true, created).expect("fijado");
        }
        for _ in 0..((seed >> 50) % 4) {
            store.record_paste(id, created + 60_000).expect("pegado");
        }
        if at % 5_000 == 0 && at > 0 {
            println!("  {at} ítems en {:.1?}", started.elapsed());
        }
    }
    store.checkpoint().expect("checkpoint");
    let db_size = std::fs::metadata(root.join("history.db"))
        .map(|m| m.len())
        .unwrap_or(0);
    println!(
        "{count} ítems ({images} imágenes con miniatura, {files} archivos) en {:.1?} → {} · history.db {:.1} MB",
        started.elapsed(),
        root.display(),
        db_size as f64 / 1_048_576.0
    );
}

fn text_at(at: usize, seed: u64) -> String {
    if at.is_multiple_of(10) {
        return SPECIAL[(seed >> 20) as usize % SPECIAL.len()].to_owned();
    }
    let sentences = 1 + (seed >> 24) as usize % 12;
    let mut text = String::new();
    for n in 0..sentences {
        if n > 0 {
            text.push(' ');
        }
        text.push_str(SENTENCES[(at + n * 5) % SENTENCES.len()]);
    }
    text.push_str(&format!(" ({at})"));
    text
}

fn text_item(text: &str) -> Item {
    Item {
        kind: Some(kind::classify_text(text)),
        formats: vec![Format {
            id: SYNTHETIC_TEXT.into(),
            payload: Payload::stored(text.as_bytes().to_vec()),
        }],
    }
}

fn folder_path_at(at: usize, seed: u64) -> String {
    const FOLDERS: [&str; 6] = [
        "Proyectos",
        "Facturas 2026",
        "Capturas",
        "Documentos de trabajo",
        "Música",
        "Respaldos",
    ];
    let name = FOLDERS[(seed >> 44) as usize % FOLDERS.len()];
    if cfg!(windows) {
        format!("C:\\Users\\Mario\\Documentos\\{at}\\{name}")
    } else {
        format!("/Users/mario/Documentos/{at}/{name}")
    }
}

fn file_path_at(at: usize, seed: u64) -> String {
    const NAMES: [&str; 8] = [
        "informe.pdf",
        "foto ñ.jpg",
        "grabación.mp4",
        "canción.flac",
        "presupuesto.xlsx",
        "notas.txt",
        "captura.png",
        "instalador.msi",
    ];
    let name = NAMES[(seed >> 44) as usize % NAMES.len()];
    if cfg!(windows) {
        format!("C:\\Users\\Mario\\Documentos\\{at}\\{name}")
    } else {
        format!("/Users/mario/Documentos/{at}/{name}")
    }
}

fn thumbnail_png(at: usize, seed: u64) -> Vec<u8> {
    let mut image = image::RgbaImage::new(THUMB_SIDE, THUMB_SIDE);
    let base = [
        (seed >> 8) as u8 % 200 + 40,
        (seed >> 16) as u8 % 200 + 40,
        (seed >> 24) as u8 % 200 + 40,
    ];
    for (x, y, px) in image.enumerate_pixels_mut() {
        let band = (y / 32).is_multiple_of(2);
        let shade = if band { 0u8 } else { 24 };
        let card = x > 20 && x < THUMB_SIDE - 20 && y > 40 && y < THUMB_SIDE - 40;
        *px = if card {
            let line = (y / 12) % 3 == 1 && x % 96 < 60 + (at % 30) as u32;
            if line {
                image::Rgba([30, 30, 40, 255])
            } else {
                image::Rgba([245, 245, 250, 255])
            }
        } else {
            image::Rgba([
                base[0].saturating_sub(shade),
                base[1].saturating_sub(shade),
                base[2].saturating_sub(shade),
                255,
            ])
        };
    }
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("png");
    out.into_inner()
}
