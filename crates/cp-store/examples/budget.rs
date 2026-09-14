use cp_core::item::Item;
use cp_store::Store;
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
            store.search("factur").expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "búsqueda que no encuentra nada",
            ceiling: Duration::from_millis(20),
        },
        || {
            store.search("inexistente").expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "búsqueda con acentos y ligaduras",
            ceiling: Duration::from_millis(20),
        },
        || {
            store.search("straße").expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "peor caso: casa con todo el historial",
            ceiling: Duration::from_millis(60),
        },
        || {
            store.search("con").expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "página diez por OFFSET (lo que no se usa)",
            ceiling: Duration::from_millis(40),
        },
        || {
            store.search_page("con", 100, 1000).expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "página diez por cursor",
            ceiling: Duration::from_millis(20),
        },
        || {
            store
                .search_after("factura", 100, Some(48_000))
                .expect("consulta");
        },
    );
    over += measure(
        Budget {
            what: "página muy profunda por cursor",
            ceiling: Duration::from_millis(20),
        },
        || {
            store
                .search_after("elemento", 100, Some(200))
                .expect("consulta");
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
