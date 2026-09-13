use cp_core::item::{Item, Payload};
use cp_core::search::fold;
use rusqlite::{Connection, OptionalExtension, Result, params};

pub struct Store {
    db: Connection,
}

impl Store {
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
            "INSERT INTO items (uuid, kind, preview_text, created_at, content_hash, search_text)
             VALUES (?1, 'text', ?2, ?3, ?4, ?5)",
            params![uuid, text, created_at, hash, fold(text)],
        )?;
        Ok(self.db.last_insert_rowid())
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
        let hash = cp_core::hash::content_hash(preview.as_bytes()) as i64;
        let kind = item.kind.map(|k| format!("{k:?}").to_lowercase());
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, content_hash, search_text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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
        self.db
            .query_row(
                "SELECT id FROM items WHERE content_hash = ?1 LIMIT 1",
                [hash],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn formats_of(&self, id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT format FROM item_formats WHERE item_id = ?1 ORDER BY format")?;
        let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
        rows.collect()
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
        self.db.execute(
            "DELETE FROM items
             WHERE broken_since IS NOT NULL AND broken_since < ?1 AND pinned = 0",
            [cutoff],
        )
    }

    pub fn pin(&self, id: i64) -> Result<()> {
        self.db
            .execute("UPDATE items SET pinned = 1 WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64> {
        self.db
            .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
    }

    pub fn search(&self, query: &str) -> Result<Vec<String>> {
        let folded = fold(query);
        let mut stmt = self.db.prepare(
            "SELECT items.preview_text
             FROM items_fts
             JOIN items ON items.id = items_fts.rowid
             WHERE items_fts MATCH ?1
             ORDER BY items.created_at DESC",
        )?;
        let rows = stmt.query_map([format!("{folded}*")], |row| row.get::<_, String>(0))?;
        rows.collect()
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
            kind: Some(cp_core::formats::Kind::Text),
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

    #[test]
    fn a_word_that_is_not_there_finds_nothing() {
        let store = seeded();
        assert!(store.search("berlin").expect("consulta").is_empty());
    }
}
