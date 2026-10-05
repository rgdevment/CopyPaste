#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Root {
    Object,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub root: Root,
    pub named: Vec<String>,
    pub pairs: Vec<String>,
    pub counted: usize,
    pub deep: usize,
    pub whole: bool,
}

const NAMES_UP_TO: usize = 8;
const VALUE_ROOM: usize = 24;

fn summary_of(raw: &str) -> String {
    let raw = raw.trim();
    match raw.chars().next() {
        Some('{') => "{…}".to_owned(),
        Some('[') => "[…]".to_owned(),
        _ if raw.chars().count() > VALUE_ROOM => {
            let kept: String = raw.chars().take(VALUE_ROOM).collect();
            format!("{kept}…")
        }
        _ => raw.to_owned(),
    }
}

pub fn shape_of(text: &str) -> Option<Shape> {
    let mut chars = text
        .char_indices()
        .skip_while(|(_, one)| one.is_whitespace());
    let (_, opened) = chars.next()?;
    let root = match opened {
        '{' => Root::Object,
        '[' => Root::Array,
        _ => return None,
    };

    let mut shape = Shape {
        root,
        named: Vec::new(),
        pairs: Vec::new(),
        counted: 0,
        deep: 1,
        whole: false,
    };
    let mut depth = 1usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut word = String::new();
    let mut held: Option<String> = None;
    let mut body = false;
    let mut taking: Option<(Option<String>, usize)> = match root {
        Root::Array => Some((
            None,
            opened.len_utf8() + text.len() - text.trim_start().len(),
        )),
        Root::Object => None,
    };
    let take = |taking: &mut Option<(Option<String>, usize)>, until: usize, shape: &mut Shape| {
        if let Some((name, from)) = taking.take() {
            let raw = text.get(from..until).unwrap_or_default();
            if raw.trim().is_empty() || shape.pairs.len() >= NAMES_UP_TO {
                return;
            }
            let value = summary_of(raw);
            shape.pairs.push(match name {
                Some(name) => format!("{name}: {value}"),
                None => value,
            });
        }
    };

    for (at, one) in chars {
        if quoted {
            if escaped {
                escaped = false;
                word.push(one);
                continue;
            }
            match one {
                '\\' => escaped = true,
                '"' => {
                    quoted = false;
                    held = Some(std::mem::take(&mut word));
                }
                _ => word.push(one),
            }
            continue;
        }
        match one {
            '"' => {
                quoted = true;
                body = true;
                word.clear();
            }
            '{' | '[' => {
                depth += 1;
                shape.deep = shape.deep.max(depth);
                body = true;
                held = None;
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                held = None;
                if depth == 0 {
                    take(&mut taking, at, &mut shape);
                    if root == Root::Array && body {
                        shape.counted += 1;
                    }
                    shape.whole = true;
                    break;
                }
            }
            ':' if depth == 1 && root == Root::Object => {
                if let Some(name) = held.take() {
                    shape.counted += 1;
                    if shape.named.len() < NAMES_UP_TO {
                        shape.named.push(name.clone());
                    }
                    taking = Some((Some(name), at + 1));
                }
            }
            ',' if depth == 1 => {
                held = None;
                take(&mut taking, at, &mut shape);
                if root == Root::Array {
                    taking = Some((None, at + 1));
                }
                if root == Root::Array {
                    shape.counted += 1;
                    body = false;
                }
            }
            one if !one.is_whitespace() => {
                body = true;
                held = None;
            }
            _ => {}
        }
    }

    if !shape.whole && root == Root::Array && body {
        shape.counted += 1;
    }
    if !shape.whole
        && let Some((name, from)) = taking.take()
    {
        let raw = text.get(from..).unwrap_or_default().trim();
        if !raw.is_empty() && shape.pairs.len() < NAMES_UP_TO {
            let cut: String = raw.chars().take(VALUE_ROOM).collect();
            let value = if raw.starts_with('{') {
                "{…".to_owned()
            } else if raw.starts_with('[') {
                "[…".to_owned()
            } else {
                format!("{cut}…")
            };
            shape.pairs.push(match name {
                Some(name) => format!("{name}: {value}"),
                None => value,
            });
        }
    }
    Some(shape)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    pub root: String,
    pub counted: String,
    pub keys: String,
    pub pairs: String,
}

pub fn said_of(text: &str, english: bool) -> Option<Said> {
    let shape = shape_of(text)?;
    Some(Said {
        root: match shape.root {
            Root::Object => "{ }".to_owned(),
            Root::Array => "[ ]".to_owned(),
        },
        counted: counted_text(&shape, english),
        keys: shape.named.join(" · "),
        pairs: shape.pairs.join(", "),
    })
}

fn counted_text(shape: &Shape, english: bool) -> String {
    let say = |es: &'static str, en: &'static str| crate::say::pick_in(english, es, en);
    let what = match (shape.root, shape.counted) {
        (Root::Object, 1) => say("clave", "key"),
        (Root::Object, _) => say("claves", "keys"),
        (Root::Array, 1) => say("elemento", "item"),
        (Root::Array, _) => say("elementos", "items"),
    };
    let plus = if shape.whole { "" } else { "+" };
    let how_many = format!("{}{plus} {what}", shape.counted);
    if shape.deep <= 1 {
        return how_many;
    }
    if english {
        format!("{how_many}, {} levels deep", shape.deep)
    } else {
        format!("{how_many} en {} niveles", shape.deep)
    }
}

#[cfg(test)]
#[path = "shape_test.rs"]
mod tests;
