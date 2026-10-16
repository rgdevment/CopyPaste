pub fn tolerance(typed: &str) -> usize {
    match typed.chars().count() {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    }
}

pub fn distance(typed: &str, word: &str, prefix: bool) -> Option<usize> {
    let limit = tolerance(typed);
    if limit == 0 {
        return None;
    }
    let a: Vec<char> = typed.chars().collect();
    let b: Vec<char> = word.chars().collect();
    let reach = if prefix {
        b.len().min(a.len() + limit)
    } else {
        b.len()
    };
    if reach + limit < a.len() || reach > a.len() + limit {
        return None;
    }
    let b = &b[..reach];
    let mut rows = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (j, cell) in rows[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        rows[i][0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (rows[i - 1][j] + 1)
                .min(rows[i][j - 1] + 1)
                .min(rows[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(rows[i - 2][j - 2] + 1);
            }
            rows[i][j] = best;
        }
    }
    let last = &rows[a.len()];
    let found = if prefix {
        let from = a.len().saturating_sub(limit).min(b.len());
        last[from..].iter().copied().min()?
    } else {
        last[b.len()]
    };
    (found <= limit).then_some(found)
}

#[cfg(test)]
#[path = "typo_test.rs"]
mod tests;
