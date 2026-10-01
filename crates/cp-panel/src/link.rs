#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parts {
    pub domain: String,
    pub path: String,
}

pub fn parts_of(preview: &str) -> Option<Parts> {
    let first = preview.lines().next()?.trim();
    let after = first
        .split_once("://")
        .map_or(first, |(_, rest)| rest)
        .trim_start_matches('/');
    if after.is_empty() {
        return None;
    }
    let (host, rest) = match after.find(['/', '?', '#']) {
        Some(at) => (&after[..at], &after[at..]),
        None => (after, ""),
    };
    let host = host.split('@').next_back().unwrap_or(host);
    let bare = host.split(':').next().unwrap_or(host);
    let bare = bare.trim_start_matches("www.");
    if bare.is_empty() || !bare.contains('.') {
        return None;
    }
    Some(Parts {
        domain: bare.to_lowercase(),
        path: rest.to_owned(),
    })
}

#[cfg(test)]
#[path = "link_test.rs"]
mod tests;
