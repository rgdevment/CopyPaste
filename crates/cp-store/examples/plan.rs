use std::time::Instant;

fn main() {
    let store = cp_store::Store::in_memory().unwrap();
    for at in 0..50_000 {
        store
            .insert_text(&format!("u{at}"), &format!("elemento {at} con texto"), at)
            .unwrap();
    }
    let db = store.raw();

    let variants = [
        (
            "join + order by items (la actual)",
            "SELECT items.modified_at, items.preview_text
             FROM items_fts JOIN items ON items.id = items_fts.rowid
             WHERE items_fts MATCH ?1 AND (?2 IS NULL OR items.modified_at < ?2)
             ORDER BY items.modified_at DESC LIMIT 100",
        ),
        (
            "IN (subconsulta fts)",
            "SELECT modified_at, preview_text FROM items
             WHERE id IN (SELECT rowid FROM items_fts WHERE items_fts MATCH ?1)
               AND (?2 IS NULL OR modified_at < ?2)
             ORDER BY modified_at DESC LIMIT 100",
        ),
        (
            "order by rowid del fts",
            "SELECT items.modified_at, items.preview_text
             FROM items_fts JOIN items ON items.id = items_fts.rowid
             WHERE items_fts MATCH ?1 AND (?2 IS NULL OR items.modified_at < ?2)
             ORDER BY items_fts.rowid DESC LIMIT 100",
        ),
    ];

    for (name, sql) in variants {
        let mut plan = db.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let steps: Vec<String> = plan
            .query_map(rusqlite::params!["\"element\"*", None::<i64>], |r| {
                r.get::<_, String>(3)
            })
            .unwrap()
            .map(Result::unwrap)
            .collect();

        let mut stmt = db.prepare(sql).unwrap();
        let mut taken = Vec::new();
        for _ in 0..7 {
            let at = Instant::now();
            let rows: Vec<(i64, String)> = stmt
                .query_map(rusqlite::params!["\"element\"*", None::<i64>], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect();
            assert_eq!(rows.len(), 100);
            taken.push(at.elapsed());
        }
        taken.sort_unstable();
        println!("\n{name}\n   p50 {:?}", taken[taken.len() / 2]);
        for step in steps {
            println!("   · {step}");
        }
    }
}
