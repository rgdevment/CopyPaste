use crate::{Error, Result};
use cp_core::item::{Item, Payload};
use cp_core::search::fold;
use rusqlite::{Connection, OptionalExtension, params};

/// Lo que el usuario escribe no es sintaxis FTS5, y tratarlo como si lo fuera
/// rompe la búsqueda con una comilla, un asterisco o un guion delante —y con
/// el buscador vacío, que ocurre cada vez que se borra lo escrito—. Cada
/// palabra se envuelve entre comillas para que FTS la lea como texto literal,
/// y el prefijo se pide fuera de ellas.
fn fts_expression(folded: &str) -> Option<String> {
    let terms: Vec<String> = folded
        .split_whitespace()
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .map(|word| format!("\"{}\"*", word.replace('"', "\"\"")))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

pub struct Store {
    db: Connection,
}

impl Store {
    /// Solo para diagnóstico: mirar planes de consulta desde un ejemplo.
    pub fn raw(&self) -> &Connection {
        &self.db
    }

    pub fn in_memory() -> Result<Self> {
        let db = Connection::open_in_memory()?;
        crate::schema::create(&db)?;
        Ok(Self { db })
    }

    /// El texto se normaliza **al escribir**, con la misma función que
    /// normaliza el término al buscar. Ese es el invariante que hoy falta: la
    /// 2.x normaliza solo el término, así que `Straße` no se encuentra ni
    /// escribiendo `strasse` ni escribiendo `Straße`.
    pub fn insert_text(&self, uuid: &str, text: &str, created_at: i64) -> Result<i64> {
        let hash = cp_core::hash::content_hash(text.as_bytes()) as i64;
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                                content_hash, search_text)
             VALUES (?1, 'text', ?2, ?3, ?3, ?3, ?4, ?5)",
            params![uuid, text, created_at, hash, fold(text)],
        )?;
        Ok(self.db.last_insert_rowid())
    }

    /// Volver a copiar algo que ya estaba lo sube en la lista, no lo duplica.
    /// La 2.x ordena por `modified_at` justamente por esto.
    pub fn touch(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items
             SET modified_at = ?2, last_used_at = ?2, updated_at = ?2,
                 paste_count = paste_count + 1
             WHERE id = ?1",
            params![id, at],
        )?;
        Ok(())
    }

    /// El texto que Vision leyó dentro de una imagen.
    ///
    /// Va en su propia columna del índice y no en `search_text` para que se
    /// pueda distinguir «lo que el usuario copió» de «lo que había escrito en
    /// la imagen», y para poder rehacerlo sin tocar lo demás. Se escribe
    /// después de capturar, nunca durante: reconocer texto cuesta unos 180 ms
    /// y el camino de captura no puede pagarlos.
    pub fn set_ocr_text(&self, id: i64, text: &str, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET search_ocr = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, fold(text), at],
        )?;
        Ok(())
    }

    /// Los ítems de imagen a los que todavía no se les ha pasado el OCR.
    pub fn pending_ocr(&self, limit: usize) -> Result<Vec<i64>> {
        let mut stmt = self.db.prepare(
            "SELECT id FROM items
             WHERE kind = 'image' AND search_ocr = '' AND deleted_at IS NULL
             ORDER BY modified_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// La etiqueta entra en el índice, así que se busca por ella.
    pub fn set_label(&self, id: i64, label: Option<&str>, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET label = ?2, search_label = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, label, label.map(fold).unwrap_or_default(), at],
        )?;
        Ok(())
    }

    pub fn set_source(&self, id: i64, app: &str, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET app_source = ?2, search_app = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, app, fold(app), at],
        )?;
        Ok(())
    }

    /// Una tumba, no un borrado: la nube que vendrá necesita saber que algo
    /// dejó de existir, y sin esto una sincronización lo resucitaría.
    pub fn mark_deleted(&self, id: i64, at: i64) -> Result<()> {
        // La lápida guarda identidad y fechas para que la sincronización sepa
        // que esto dejó de existir. Todo lo demás se va: el contenido en claro,
        // su copia plegada en el índice, y las filas de formato con sus bytes.
        // Dejarlo era incumplir lo que PRIVACY.md promete al usuario.
        self.db.execute(
            "UPDATE items
             SET deleted_at = ?2, updated_at = ?2,
                 preview_text = '', search_text = '', search_label = '',
                 search_app = '', search_ocr = '', label = NULL, app_source = NULL,
                 thumb_path = NULL, content_hash = 0
             WHERE id = ?1",
            params![id, at],
        )?;
        self.db
            .execute("DELETE FROM item_formats WHERE item_id = ?1", [id])?;
        Ok(())
    }

    /// Lo que cambió después de un punto, que es lo que una sincronización
    /// necesita preguntar.
    pub fn changed_since(&self, version: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT uuid FROM items WHERE updated_at > ?1 ORDER BY updated_at")?;
        let rows = stmt.query_map([version], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Guarda el ítem con **todas** sus filas de formato. El `content_hash`
    /// es del contenido completo, así que dos copias iguales no crean dos
    /// ítems y una copia distinta nunca se toma por repetida.
    pub fn insert_item(
        &self,
        uuid: &str,
        item: &Item,
        preview: &str,
        created_at: i64,
    ) -> Result<i64> {
        // Mientras el almacén de blobs no exista, aceptar un `Blob` sería
        // guardar la fila con su tamaño y tirar los bytes: el ítem quedaría
        // vacío y nadie se enteraría. Es preferible negarse.
        if let Some(oversized) = item.oversized_format() {
            return Err(Error::NeedsBlobStore {
                format: oversized.0,
                size: oversized.1,
            });
        }
        let hash = item.fingerprint() as i64;
        let kind = item.kind.map(|k| k.as_str());
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                                content_hash, search_text)
             VALUES (?1, ?2, ?3, ?4, ?4, ?4, ?5, ?6)",
            params![uuid, kind, preview, created_at, hash, fold(preview)],
        )?;
        let id = self.db.last_insert_rowid();
        for format in &item.formats {
            let (inline, size) = match &format.payload {
                Payload::Inline(bytes) => (Some(bytes.clone()), Some(bytes.len() as i64)),
                Payload::Blob(bytes) => (None, Some(bytes.len() as i64)),
                Payload::TooBig { size } => (None, Some(*size as i64)),
                Payload::Announced { size } => (None, size.map(|s| s as i64)),
                Payload::Absent => (None, None),
            };
            self.db.execute(
                "INSERT INTO item_formats (item_id, format, size_bytes, inline_data)
                 VALUES (?1, ?2, ?3, ?4)",
                params![id, format.id, size, inline],
            )?;
        }
        Ok(id)
    }

    pub fn find_by_hash(&self, content: &str) -> Result<Option<i64>> {
        let hash = cp_core::hash::content_hash(content.as_bytes()) as i64;
        Ok(self
            .db
            .query_row(
                "SELECT id FROM items WHERE content_hash = ?1 AND deleted_at IS NULL LIMIT 1",
                [hash],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn formats_of(&self, id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT format FROM item_formats WHERE item_id = ?1 ORDER BY format")?;
        let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Un ítem cuyo archivo ya no existe **no se borra**: se marca. La
    /// diferencia entre «se me borró el historial» y «esto ya no está en el
    /// disco» la nota el usuario de inmediato.
    pub fn mark_broken(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET broken_since = ?2 WHERE id = ?1 AND broken_since IS NULL",
            params![id, at],
        )?;
        Ok(())
    }

    /// Los rotos se van cuando cumplen su plazo, nunca antes. Un ítem fijado
    /// es una decisión del usuario y la limpieza no la revoca.
    pub fn purge_broken_before(&self, cutoff: i64) -> Result<usize> {
        Ok(self.db.execute(
            "DELETE FROM items
             WHERE broken_since IS NOT NULL AND broken_since < ?1 AND pinned = 0",
            [cutoff],
        )?)
    }

    pub fn pin(&self, id: i64) -> Result<()> {
        self.db
            .execute("UPDATE items SET pinned = 1 WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Cuántos ítems tiene el usuario. Las lápidas no se cuentan: para él
    /// están borradas.
    pub fn count(&self) -> Result<i64> {
        Ok(self.db.query_row(
            "SELECT COUNT(*) FROM items WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?)
    }

    /// Cuántas filas pide una lista antes de que el usuario haga scroll. Sin
    /// tope, una consulta que casa con todo materializa el historial entero
    /// en cada pulsación.
    pub const PAGE: usize = 100;

    pub fn search(&self, query: &str) -> Result<Vec<String>> {
        self.search_page(query, Self::PAGE, 0)
    }

    /// Página siguiente a partir del último visto, en vez de `OFFSET`.
    ///
    /// Con `OFFSET`, SQLite ordena el resultado entero y descarta lo saltado,
    /// así que la página diez cuesta diez veces la primera. Con el corte por
    /// `modified_at` puede recorrer el índice de recencia y parar al llenar
    /// la página: cada página cuesta lo mismo que la primera.
    pub fn search_after(
        &self,
        query: &str,
        limit: usize,
        after: Option<i64>,
    ) -> Result<Vec<(i64, String)>> {
        let Some(expression) = fts_expression(&fold(query)) else {
            return Ok(Vec::new());
        };
        let mut stmt = self.db.prepare(
            "SELECT items.modified_at, items.preview_text
             FROM items_fts
             JOIN items ON items.id = items_fts.rowid
             WHERE items_fts MATCH ?1
               AND items.deleted_at IS NULL
               AND (?2 IS NULL OR items.modified_at < ?2)
             ORDER BY items.modified_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(rusqlite::params![expression, after, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn search_page(&self, query: &str, limit: usize, offset: usize) -> Result<Vec<String>> {
        let Some(expression) = fts_expression(&fold(query)) else {
            return Ok(Vec::new());
        };
        let mut stmt = self.db.prepare(
            "SELECT items.preview_text
             FROM items_fts
             JOIN items ON items.id = items_fts.rowid
             WHERE items_fts MATCH ?1 AND items.deleted_at IS NULL
             ORDER BY items.modified_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![expression, limit as i64, offset as i64],
            |row| row.get::<_, String>(0),
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cp_core::item::Format;

    fn seeded() -> Store {
        let store = Store::in_memory().expect("esquema");
        for (at, text) in [
            "el café de la esquina",
            "Straße Hauptbahnhof",
            "encyclopædia britannica",
            "Łódź centrum",
            "Peçanha e Gonçalves",
        ]
        .iter()
        .enumerate()
        {
            store
                .insert_text(&format!("uuid-{at}"), text, at as i64)
                .expect("insert");
        }
        store
    }

    /// Los cuatro casos del fallo verificado con SQLite en la 2.x, donde solo
    /// el primero funcionaba —y por suerte, gracias al tokenizer—.
    #[test]
    fn the_four_cases_that_2x_gets_wrong() {
        let store = seeded();
        for (query, expected) in [
            ("cafe", "el café de la esquina"),
            ("strasse", "Straße Hauptbahnhof"),
            ("encyclopaedia", "encyclopædia britannica"),
            ("lodz", "Łódź centrum"),
        ] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit == expected),
                "buscando «{query}» no apareció «{expected}»: {hits:?}"
            );
        }
    }

    /// Y en la otra dirección, que es la mitad que nadie prueba: escribir la
    /// palabra con el carácter especial también tiene que encontrarla.
    #[test]
    fn it_works_in_both_directions() {
        let store = seeded();
        for (query, expected) in [
            ("café", "el café de la esquina"),
            ("Straße", "Straße Hauptbahnhof"),
            ("encyclopædia", "encyclopædia britannica"),
            ("Łódź", "Łódź centrum"),
            ("Gonçalves", "Peçanha e Gonçalves"),
        ] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit == expected),
                "buscando «{query}» no apareció «{expected}»: {hits:?}"
            );
        }
    }

    #[test]
    fn the_stored_text_keeps_its_accents() {
        let store = seeded();
        let hits = store.search("cafe").expect("consulta");
        assert_eq!(
            hits.first().map(String::as_str),
            Some("el café de la esquina"),
            "se busca sin tildes pero se muestra como se copió"
        );
    }

    fn sample_item() -> Item {
        Item {
            kind: Some(cp_core::kind::Kind::Text),
            formats: vec![
                Format {
                    id: "public.utf8-plain-text".into(),
                    payload: Payload::Inline(b"hola".to_vec()),
                },
                Format {
                    id: "public.rtf".into(),
                    payload: Payload::Inline(vec![0u8; 400]),
                },
                Format {
                    id: "com.apple.icns".into(),
                    payload: Payload::Announced { size: None },
                },
                Format {
                    id: "fndf".into(),
                    payload: Payload::Absent,
                },
            ],
        }
    }

    #[test]
    fn an_item_too_big_for_the_row_is_refused_not_emptied() {
        let store = Store::in_memory().expect("esquema");
        let big = Item {
            kind: None,
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Blob(vec![0u8; 100_000]),
            }],
        };
        assert!(
            store.insert_item("uuid-grande", &big, "", 1).is_err(),
            "mejor negarse que guardar un ítem sin sus bytes"
        );
        assert_eq!(store.count().expect("cuenta"), 0);
    }

    #[test]
    fn two_images_with_no_preview_are_two_items() {
        let store = Store::in_memory().expect("esquema");
        let image = |byte: u8| Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![byte; 512]),
            }],
        };
        store
            .insert_item("uuid-a", &image(1), "", 1)
            .expect("insert");
        store
            .insert_item("uuid-b", &image(2), "", 2)
            .expect("insert");
        let hashes: Vec<i64> = store
            .db
            .prepare("SELECT content_hash FROM items ORDER BY id")
            .expect("prepara")
            .query_map([], |row| row.get(0))
            .expect("consulta")
            .map(|row| row.expect("fila"))
            .collect();
        assert_ne!(
            hashes[0], hashes[1],
            "hashear el preview vacío las haría la misma"
        );
    }

    #[test]
    fn an_item_keeps_every_format_it_was_offered() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_item("uuid-multi", &sample_item(), "hola", 1)
            .expect("insert");
        let formats = store.formats_of(id).expect("formatos");
        assert_eq!(formats.len(), 4, "las cuatro filas, incluidas las vacías");
        assert!(formats.contains(&"com.apple.icns".to_string()));
        assert!(formats.contains(&"fndf".to_string()));
    }

    #[test]
    fn the_same_content_is_found_by_its_hash() {
        let store = Store::in_memory().expect("esquema");
        store.insert_text("uuid-a", "repetido", 1).expect("insert");
        assert!(store.find_by_hash("repetido").expect("busca").is_some());
        assert!(store.find_by_hash("distinto").expect("busca").is_none());
    }

    #[test]
    fn deleting_an_item_takes_its_formats_with_it() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_item("uuid-cascade", &sample_item(), "hola", 1)
            .expect("insert");
        store.mark_broken(id, 10).expect("marca");
        store.purge_broken_before(20).expect("purga");
        assert_eq!(store.formats_of(id).expect("formatos").len(), 0);
    }

    #[test]
    fn a_broken_item_survives_until_its_time_is_up() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-roto", "archivo ido", 1)
            .expect("insert");
        store.mark_broken(id, 100).expect("marca");
        assert_eq!(store.purge_broken_before(50).expect("purga"), 0);
        assert_eq!(store.count().expect("cuenta"), 1, "aún no cumple el plazo");
        assert_eq!(store.purge_broken_before(150).expect("purga"), 1);
        assert_eq!(store.count().expect("cuenta"), 0);
    }

    #[test]
    fn marking_a_broken_item_twice_does_not_restart_its_clock() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-roto", "archivo ido", 1)
            .expect("insert");
        store.mark_broken(id, 100).expect("primera");
        store.mark_broken(id, 900).expect("segunda");
        assert_eq!(
            store.purge_broken_before(150).expect("purga"),
            1,
            "vale la primera vez que se vio roto, no la última"
        );
    }

    #[test]
    fn a_pinned_item_is_never_purged_even_when_broken() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-fijado", "importante", 1)
            .expect("insert");
        store.pin(id).expect("fija");
        store.mark_broken(id, 100).expect("marca");
        assert_eq!(store.purge_broken_before(9999).expect("purga"), 0);
        assert_eq!(store.count().expect("cuenta"), 1);
    }

    /// Lo que un usuario puede escribir en el buscador sin querer decir nada
    /// especial. Ninguna de estas puede devolver un error de SQL.
    #[test]
    fn no_query_a_person_can_type_breaks_the_search() {
        let store = seeded();
        for query in [
            "\"",
            "\"\"",
            "*",
            "(",
            ")",
            "()",
            "a AND b",
            "NOT café",
            "search_text:café",
            "NEAR(a b)",
            "-café",
            "^café",
            "café*",
            "{café}",
            "[café]",
            "café OR",
            "OR",
            "AND OR NOT",
            "",
            " ",
            "\t\n",
            "...",
            "!!!",
            "\\",
            "%",
            "_",
            "'; DROP TABLE items; --",
        ] {
            store
                .search(query)
                .unwrap_or_else(|why| panic!("«{query}» rompió la búsqueda: {why}"));
        }
    }

    #[test]
    fn an_empty_search_returns_nothing_rather_than_everything() {
        let store = seeded();
        for empty in ["", "   ", "\t", "-", "!!", "***"] {
            assert!(
                store.search(empty).expect("consulta").is_empty(),
                "«{empty}» debería no devolver nada"
            );
        }
    }

    #[test]
    fn the_punctuation_around_a_word_does_not_hide_it() {
        let store = seeded();
        for query in ["-café", "^café", "(café)", "«café»", "café!"] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit.contains("café")),
                "«{query}» no encontró el café"
            );
        }
    }

    #[test]
    fn scripts_that_are_not_latin_go_in_and_come_out() {
        let store = Store::in_memory().expect("esquema");
        for (at, text) in [
            "日本語のテキスト",
            "Привет мир",
            "مرحبا بالعالم",
            "🎉 fiesta 🎊",
            "한국어 텍스트",
        ]
        .iter()
        .enumerate()
        {
            store
                .insert_text(&format!("uuid-{at}"), text, at as i64)
                .expect("insert");
        }
        for (query, expected) in [
            ("日本語", "日本語のテキスト"),
            ("Привет", "Привет мир"),
            ("fiesta", "🎉 fiesta 🎊"),
            ("한국어", "한국어 텍스트"),
        ] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit == expected),
                "buscando «{query}» faltó «{expected}»: {hits:?}"
            );
        }
    }

    #[test]
    fn a_very_long_text_is_stored_and_found() {
        let store = Store::in_memory().expect("esquema");
        let long = format!(
            "{} aguja {}",
            "paja ".repeat(50_000),
            "paja ".repeat(50_000)
        );
        store.insert_text("uuid-largo", &long, 1).expect("insert");
        assert_eq!(store.search("aguja").expect("consulta").len(), 1);
    }

    #[test]
    fn the_same_uuid_twice_is_refused_not_duplicated() {
        let store = Store::in_memory().expect("esquema");
        store
            .insert_text("uuid-unico", "primero", 1)
            .expect("insert");
        assert!(
            store.insert_text("uuid-unico", "segundo", 2).is_err(),
            "el uuid es único por contrato"
        );
        assert_eq!(store.count().expect("cuenta"), 1);
    }

    #[test]
    fn marking_an_item_that_does_not_exist_is_not_a_failure() {
        let store = Store::in_memory().expect("esquema");
        store.mark_broken(9999, 1).expect("no existe, no pasa nada");
        assert_eq!(store.count().expect("cuenta"), 0);
    }

    #[test]
    fn a_purge_with_nothing_to_purge_removes_nothing() {
        let store = seeded();
        let before = store.count().expect("cuenta");
        assert_eq!(store.purge_broken_before(-1).expect("purga"), 0);
        assert_eq!(store.purge_broken_before(i64::MAX).expect("purga"), 0);
        assert_eq!(store.count().expect("cuenta"), before);
    }

    #[test]
    fn an_item_with_no_formats_at_all_is_still_an_item() {
        let store = Store::in_memory().expect("esquema");
        let empty = Item {
            kind: None,
            formats: vec![],
        };
        let id = store
            .insert_item("uuid-vacio", &empty, "", 1)
            .expect("insert");
        assert_eq!(store.formats_of(id).expect("formatos").len(), 0);
        assert_eq!(store.count().expect("cuenta"), 1);
    }

    #[test]
    fn a_search_that_matches_everything_still_returns_one_page() {
        let store = Store::in_memory().expect("esquema");
        for at in 0..250 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("comun {at}"), at)
                .expect("insert");
        }
        assert_eq!(store.search("comun").expect("consulta").len(), Store::PAGE);
        assert_eq!(
            store.search_page("comun", 10, 0).expect("consulta").len(),
            10
        );
    }

    #[test]
    fn paging_walks_the_whole_result_without_repeating() {
        let store = Store::in_memory().expect("esquema");
        for at in 0..25 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("pagina {at}"), at)
                .expect("insert");
        }
        let first = store.search_page("pagina", 10, 0).expect("consulta");
        let second = store.search_page("pagina", 10, 10).expect("consulta");
        let last = store.search_page("pagina", 10, 20).expect("consulta");
        assert_eq!((first.len(), second.len(), last.len()), (10, 10, 5));
        assert!(
            first.iter().all(|one| !second.contains(one)),
            "las páginas no pueden solaparse"
        );
    }

    #[test]
    fn the_cursor_walks_the_result_without_repeating_or_skipping() {
        let store = Store::in_memory().expect("esquema");
        for at in 0..25 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("cursor {at}"), at)
                .expect("insert");
        }
        let mut seen = Vec::new();
        let mut after = None;
        loop {
            let page = store.search_after("cursor", 10, after).expect("consulta");
            if page.is_empty() {
                break;
            }
            after = page.last().map(|(at, _)| *at);
            seen.extend(page.into_iter().map(|(_, text)| text));
        }
        assert_eq!(seen.len(), 25, "recorrió todo");
        let mut unique = seen.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 25, "sin repetir");
    }

    #[test]
    fn a_cursor_past_the_oldest_item_is_empty() {
        let store = seeded();
        assert!(
            store
                .search_after("café", 10, Some(-1))
                .expect("consulta")
                .is_empty()
        );
    }

    #[test]
    fn an_offset_past_the_end_is_empty_not_an_error() {
        let store = seeded();
        assert!(
            store
                .search_page("café", 10, 9999)
                .expect("consulta")
                .is_empty()
        );
    }

    #[test]
    fn copying_something_again_lifts_it_instead_of_duplicating_it() {
        let store = Store::in_memory().expect("esquema");
        let first = store.insert_text("uuid-a", "lo viejo", 10).expect("insert");
        store.insert_text("uuid-b", "lo nuevo", 20).expect("insert");

        let before = store.search("lo").expect("consulta");
        assert_eq!(before.first().map(String::as_str), Some("lo nuevo"));

        store.touch(first, 30).expect("recopiado");
        let after = store.search("lo").expect("consulta");
        assert_eq!(
            after.first().map(String::as_str),
            Some("lo viejo"),
            "recopiar algo lo sube al principio"
        );
    }

    #[test]
    fn a_label_can_be_searched_for() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-etq", "un texto cualquiera", 1)
            .expect("insert");
        assert!(store.search("factura").expect("consulta").is_empty());
        store
            .set_label(id, Some("Factura Mayo"), 2)
            .expect("etiqueta");
        let hits = store.search("factura").expect("consulta");
        assert_eq!(hits.len(), 1, "la etiqueta entra en el índice");
    }

    #[test]
    fn the_source_application_can_be_searched_for() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-app", "algo copiado", 1)
            .expect("insert");
        store.set_source(id, "Safari", 2).expect("origen");
        assert_eq!(store.search("safari").expect("consulta").len(), 1);
    }

    #[test]
    fn a_label_with_accents_is_found_without_them() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-tilde", "contenido", 1)
            .expect("insert");
        store
            .set_label(id, Some("Reunión Diseño"), 2)
            .expect("etiqueta");
        assert_eq!(store.search("reunion").expect("consulta").len(), 1);
        assert_eq!(store.search("diseño").expect("consulta").len(), 1);
    }

    #[test]
    fn removing_a_label_takes_it_out_of_the_index() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-quita", "contenido", 1)
            .expect("insert");
        store.set_label(id, Some("temporal"), 2).expect("pone");
        assert_eq!(store.search("temporal").expect("consulta").len(), 1);
        store.set_label(id, None, 3).expect("quita");
        assert!(store.search("temporal").expect("consulta").is_empty());
    }

    #[test]
    fn deleting_hides_the_item_from_everything_the_user_can_see() {
        let store = seeded();
        let id = store
            .insert_text("uuid-secreto", "contraseña del banco", 500)
            .expect("insert");
        let before = store.count().expect("cuenta");

        store.mark_deleted(id, 600).expect("borra");

        assert_eq!(store.count().expect("cuenta"), before - 1, "deja de contar");
        assert!(
            store.search("contraseña").expect("consulta").is_empty(),
            "no puede seguir encontrándose"
        );
        assert!(
            store
                .find_by_hash("contraseña del banco")
                .expect("hash")
                .is_none(),
            "volver a copiarlo debe crear un ítem nuevo, no resucitar la lápida"
        );
    }

    #[test]
    fn a_deleted_item_leaves_no_content_behind() {
        let store = Store::in_memory().expect("esquema");
        let item = sample_item();
        let id = store
            .insert_item("uuid-borrado", &item, "texto en claro", 1)
            .expect("insert");
        store.set_label(id, Some("etiqueta"), 2).expect("etiqueta");
        store.mark_deleted(id, 3).expect("borra");

        let (preview, search, label): (String, String, Option<String>) = store
            .db
            .query_row(
                "SELECT preview_text, search_text, label FROM items WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("consulta");
        assert_eq!(preview, "", "el contenido en claro se va");
        assert_eq!(search, "", "y su copia en el índice también");
        assert_eq!(label, None);
        assert_eq!(
            store.formats_of(id).expect("formatos").len(),
            0,
            "los bytes de los formatos se van con el ítem"
        );
    }

    #[test]
    fn the_tombstone_still_tells_the_sync_what_happened() {
        let store = Store::in_memory().expect("esquema");
        let id = store.insert_text("uuid-tumba", "se va", 1).expect("insert");
        store.mark_deleted(id, 50).expect("borra");
        assert!(
            store
                .changed_since(40)
                .expect("cambios")
                .contains(&"uuid-tumba".to_string()),
            "sin esto, otra máquina lo resucita"
        );
    }

    #[test]
    fn only_what_changed_after_the_mark_is_reported() {
        let store = Store::in_memory().expect("esquema");
        let old = store
            .insert_text("uuid-viejo", "antiguo", 10)
            .expect("insert");
        store
            .insert_text("uuid-nuevo", "reciente", 100)
            .expect("insert");
        let changed = store.changed_since(50).expect("cambios");
        assert_eq!(changed, vec!["uuid-nuevo".to_string()]);

        store.touch(old, 200).expect("recopiado");
        let after = store.changed_since(50).expect("cambios");
        assert_eq!(
            after,
            vec!["uuid-nuevo".to_string(), "uuid-viejo".to_string()],
            "ordenados por versión, y el recopiado ahora cuenta"
        );
    }

    #[test]
    fn touching_an_item_counts_the_paste() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-cuenta", "pegado", 1)
            .expect("insert");
        store.touch(id, 2).expect("uno");
        store.touch(id, 3).expect("dos");
        let count: i64 = store
            .db
            .query_row("SELECT paste_count FROM items WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .expect("consulta");
        assert_eq!(count, 2);
    }

    #[test]
    fn an_image_becomes_findable_by_what_is_written_inside_it() {
        let store = Store::in_memory().expect("esquema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![137, 80, 78, 71]),
            }],
        };
        let id = store
            .insert_item("uuid-captura", &image, "", 1)
            .expect("insert");

        assert!(
            store.search("pedido").expect("consulta").is_empty(),
            "todavía no se le ha pasado el OCR"
        );
        assert_eq!(store.pending_ocr(10).expect("pendientes"), vec![id]);

        store
            .set_ocr_text(id, "Pedido AB-4417 entrega 12 marzo", 2)
            .expect("ocr");

        assert_eq!(
            store.search("pedido").expect("consulta").len(),
            1,
            "una captura de pantalla se encuentra por lo que pone dentro"
        );
        assert_eq!(store.search("AB-4417").expect("consulta").len(), 1);
        assert!(
            store.pending_ocr(10).expect("pendientes").is_empty(),
            "ya no está pendiente"
        );
    }

    #[test]
    fn the_ocr_text_is_folded_like_everything_else() {
        let store = Store::in_memory().expect("esquema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-tilde", &image, "", 1)
            .expect("insert");
        store.set_ocr_text(id, "Reunión en Múnich", 2).expect("ocr");
        assert_eq!(store.search("reunion").expect("consulta").len(), 1);
        assert_eq!(store.search("munich").expect("consulta").len(), 1);
    }

    #[test]
    fn deleting_takes_the_recognised_text_with_it() {
        let store = Store::in_memory().expect("esquema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-secreta", &image, "", 1)
            .expect("insert");
        store
            .set_ocr_text(id, "clave de recuperación 8842", 2)
            .expect("ocr");
        store.mark_deleted(id, 3).expect("borra");
        assert!(
            store.search("recuperacion").expect("consulta").is_empty(),
            "lo leído dentro de la imagen también es contenido del usuario"
        );
    }

    #[test]
    fn a_word_that_is_not_there_finds_nothing() {
        let store = seeded();
        assert!(store.search("berlin").expect("consulta").is_empty());
    }
}
