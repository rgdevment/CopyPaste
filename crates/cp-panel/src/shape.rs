#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Root {
    Object,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub root: Root,
    pub named: Vec<String>,
    pub counted: usize,
    pub deep: usize,
    pub whole: bool,
}

const NAMES_UP_TO: usize = 8;

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

    for (_, one) in chars {
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
                        shape.named.push(name);
                    }
                }
            }
            ',' if depth == 1 => {
                held = None;
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
    Some(shape)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    pub root: String,
    pub counted: String,
    pub keys: String,
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
    })
}

fn counted_text(shape: &Shape, english: bool) -> String {
    let say = |es: &'static str, en: &'static str| crate::say::pick_in(english, es, en);
    let what = match (shape.root, shape.counted) {
        (Root::Object, 1) => say("clave", "key"),
        (Root::Object, _) => say("claves", "keys"),
        (Root::Array, 1) => say("elemento", "element"),
        (Root::Array, _) => say("elementos", "elements"),
    };
    let how_many = if shape.whole {
        format!("{} {what}", shape.counted)
    } else if english {
        format!("more than {} {what}", shape.counted)
    } else {
        format!("más de {} {what}", shape.counted)
    };
    if shape.deep <= 1 {
        return how_many;
    }
    format!("{how_many} · {} {}", shape.deep, say("niveles", "levels"))
}

#[cfg(test)]
#[path = "shape_test.rs"]
mod tests;
