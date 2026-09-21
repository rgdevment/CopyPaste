use cp_core::item::Item;
use cp_store::{Cursor, Filter, Store};
use std::time::{Duration, Instant};

const ITEMS: usize = 50_000;

struct Budget {
    what: &'static str,
    ceiling: Duration,
}

fn slack() -> u32 {
    std::env::var("CP_BUDGET_SLACK")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1)
        .max(1)
}

fn main() -> std::process::ExitCode {
    let store = Store::in_memory().expect("esquema");
    if slack() > 1 {
        println!("  (techos multiplicados por {})", slack());
    }

    let vocabulary = [
        "informe",
        "factura",
        "reunión",
        "contraseña",
        "dirección",
        "teléfono",
        "proyecto",
        "cliente",
        "presupuesto",
        "contrato",
        "pedido",
        "entrega",
        "https://ejemplo.test/ruta",
        "SELECT * FROM tabla",
        "def función():",
        "Straße",
        "encyclopædia",
        "日本語",
        "correo@ejemplo.test",
        "#FF8800",
    ];
    let filling = Instant::now();
    for at in 0..ITEMS {
        let word = vocabulary[at % vocabulary.len()];
        let other = vocabulary[(at * 7) % vocabulary.len()];
        store
            .insert_text(
                &format!("uuid-{at}"),
                &format!("{word} {at} sobre {other} con algo de texto alrededor"),
                at as i64,
            )
            .expect("insert");
    }
    let filled = filling.elapsed();
    println!(
        "  {ITEMS} ítems insertados en {:?} ({:?} por ítem)",
        filled,
        filled / ITEMS as u32
    );

    let mut over = 0;
    over += measure(
        Budget {
            what: "búsqueda por prefijo",
            ceiling: Duration::from_millis(20),
        },
        || {
            store
                .list(
                    &Filter {
                        query: Some("factur".into()),
                        ..Default::default()
                    },
                    100,
                    None,
                )
                .expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "búsqueda que no encuentra nada",
            ceiling: Duration::from_millis(20),
        },
        || {
            store
                .list(
                    &Filter {
                        query: Some("inexistente".into()),
                        ..Default::default()
                    },
                    100,
                    None,
                )
                .expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "búsqueda con acentos y ligaduras",
            ceiling: Duration::from_millis(20),
        },
        || {
            store
                .list(
                    &Filter {
                        query: Some("straße".into()),
                        ..Default::default()
                    },
                    100,
                    None,
                )
                .expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "peor caso: casa con todo el historial",
            ceiling: Duration::from_millis(60),
        },
        || {
            store
                .list(
                    &Filter {
                        query: Some("con".into()),
                        ..Default::default()
                    },
                    100,
                    None,
                )
                .expect("consulta");
        },
    );
    let deep = {
        let filter = Filter {
            query: Some("con".into()),
            ..Default::default()
        };
        let mut after: Option<Cursor> = None;
        for _ in 0..10 {
            after = store.list(&filter, 100, after).expect("consulta").next;
        }
        after
    };
    over += measure(
        Budget {
            what: "página diez por cursor",
            ceiling: Duration::from_millis(20),
        },
        || {
            let filter = Filter {
                query: Some("con".into()),
                ..Default::default()
            };
            store.list(&filter, 100, deep).expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "el historial sin término, primera página",
            ceiling: Duration::from_millis(20),
        },
        || {
            store.list(&Filter::default(), 100, None).expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "las pestañas con conteo",
            ceiling: Duration::from_millis(40),
        },
        || {
            store.facets(&Filter::default()).expect("facetas");
        },
    );
    over += measure(
        Budget {
            what: "deduplicación por hash",
            ceiling: Duration::from_millis(5),
        },
        || {
            store
                .find_by_hash(&Item::plain(
                    "elemento número 5000 con algo de texto alrededor para que pese",
                ))
                .expect("busca");
        },
    );
    over += measure(
        Budget {
            what: "una inserción más",
            ceiling: Duration::from_millis(5),
        },
        || {
            let _ = store.insert_text("uuid-extra", "uno más", 999_999);
        },
    );

    println!();
    if over == 0 {
        println!("todo dentro de presupuesto");
        std::process::ExitCode::SUCCESS
    } else {
        println!("{over} operaciones fuera de presupuesto");
        std::process::ExitCode::FAILURE
    }
}

fn measure(budget: Budget, mut run: impl FnMut()) -> u32 {
    let mut taken: Vec<Duration> = (0..10)
        .map(|_| {
            let at = Instant::now();
            run();
            at.elapsed()
        })
        .collect();
    taken.sort_unstable();
    let median = taken[taken.len() / 2];
    let worst = *taken.last().expect("hay muestras");
    let ceiling = budget.ceiling * slack();
    if median <= ceiling {
        println!(
            "  ok    {:34} p50 {median:>10?}  max {worst:>10?}  (techo {ceiling:?})",
            budget.what
        );
        0
    } else {
        println!(
            "  FUERA {:34} p50 {median:>10?}  max {worst:>10?}  (techo {ceiling:?})",
            budget.what
        );
        1
    }
}
