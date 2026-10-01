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
    let store = Store::in_memory().expect("schema");
    if slack() > 1 {
        println!("  (ceilings multiplied by {})", slack());
    }

    let vocabulary = [
        "report",
        "invoice",
        "meeting",
        "password",
        "address",
        "phone",
        "project",
        "client",
        "budget",
        "contract",
        "order",
        "delivery",
        "https://example.test/path",
        "SELECT * FROM table",
        "def function():",
        "Straße",
        "encyclopædia",
        "日本語",
        "mail@example.test",
        "#FF8800",
    ];
    let filling = Instant::now();
    for at in 0..ITEMS {
        let word = vocabulary[at % vocabulary.len()];
        let other = vocabulary[(at * 7) % vocabulary.len()];
        store
            .insert_text(
                &format!("uuid-{at}"),
                &format!("{word} {at} about {other} with some text around it"),
                at as i64,
            )
            .expect("insert");
    }
    let filled = filling.elapsed();
    println!(
        "  {ITEMS} items inserted in {:?} ({:?} per item)",
        filled,
        filled / ITEMS as u32
    );

    let mut over = 0;
    over += measure(
        Budget {
            what: "prefix search",
            ceiling: Duration::from_millis(20),
        },
        || {
            store
                .list(
                    &Filter {
                        query: Some("invoic".into()),
                        ..Default::default()
                    },
                    100,
                    None,
                )
                .expect("queried");
        },
    );
    over += measure(
        Budget {
            what: "a search that finds nothing",
            ceiling: Duration::from_millis(20),
        },
        || {
            store
                .list(
                    &Filter {
                        query: Some("nonexistent".into()),
                        ..Default::default()
                    },
                    100,
                    None,
                )
                .expect("queried");
        },
    );
    over += measure(
        Budget {
            what: "search with accents and ligatures",
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
                .expect("queried");
        },
    );
    over += measure(
        Budget {
            what: "worst case: matches the whole history",
            ceiling: Duration::from_millis(60),
        },
        || {
            store
                .list(
                    &Filter {
                        query: Some("with".into()),
                        ..Default::default()
                    },
                    100,
                    None,
                )
                .expect("queried");
        },
    );
    let deep = {
        let filter = Filter {
            query: Some("with".into()),
            ..Default::default()
        };
        let mut after: Option<Cursor> = None;
        for _ in 0..10 {
            after = store.list(&filter, 100, after).expect("queried").next;
        }
        after
    };
    over += measure(
        Budget {
            what: "page ten by cursor",
            ceiling: Duration::from_millis(20),
        },
        || {
            let filter = Filter {
                query: Some("with".into()),
                ..Default::default()
            };
            store.list(&filter, 100, deep.clone()).expect("queried");
        },
    );
    over += measure(
        Budget {
            what: "the history with no term, first page",
            ceiling: Duration::from_millis(20),
        },
        || {
            store.list(&Filter::default(), 100, None).expect("queried");
        },
    );
    over += measure(
        Budget {
            what: "the tabs with their counts",
            ceiling: Duration::from_millis(40),
        },
        || {
            store.facets(&Filter::default()).expect("facets");
        },
    );
    over += measure(
        Budget {
            what: "deduplication by hash",
            ceiling: Duration::from_millis(5),
        },
        || {
            store
                .find_by_hash(&Item::plain(
                    "item number 5000 with some text around it to give it some heft",
                ))
                .expect("searched");
        },
    );
    over += measure(
        Budget {
            what: "one more insertion",
            ceiling: Duration::from_millis(5),
        },
        || {
            let _ = store.insert_text("uuid-extra", "one more", 999_999);
        },
    );

    println!();
    if over == 0 {
        println!("everything within budget");
        std::process::ExitCode::SUCCESS
    } else {
        println!("{over} operations over budget");
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
    let worst = *taken.last().expect("there are samples");
    let ceiling = budget.ceiling * slack();
    if median <= ceiling {
        println!(
            "  ok    {:34} p50 {median:>10?}  max {worst:>10?}  (ceiling {ceiling:?})",
            budget.what
        );
        0
    } else {
        println!(
            "  OVER  {:34} p50 {median:>10?}  max {worst:>10?}  (ceiling {ceiling:?})",
            budget.what
        );
        1
    }
}
