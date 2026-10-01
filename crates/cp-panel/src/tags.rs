pub fn after(chosen: &[String], key: &str, adding: bool) -> Vec<String> {
    let held = chosen.iter().position(|one| one == key);
    if adding {
        let mut next = chosen.to_vec();
        match held {
            Some(at) => {
                next.remove(at);
            }
            None => next.push(key.to_owned()),
        }
        return next;
    }
    match held {
        Some(_) if chosen.len() == 1 => Vec::new(),
        _ => vec![key.to_owned()],
    }
}

#[cfg(test)]
#[path = "tags_test.rs"]
mod tests;
