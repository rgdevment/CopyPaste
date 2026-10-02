pub const SCHEME: &str = "linkunbound";

pub fn opens_the_web(url: &str) -> bool {
    let said = url.trim_start();
    let Some((scheme, rest)) = said.split_once("://") else {
        return false;
    };
    if rest.is_empty() {
        return false;
    }
    scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
}

pub fn encoded(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    for byte in url.as_bytes() {
        if byte.is_ascii_alphanumeric() {
            out.push(char::from(*byte));
        } else {
            out.push('%');
            out.push(hex(byte >> 4));
            out.push(hex(byte & 0x0F));
        }
    }
    out
}

fn hex(nibble: u8) -> char {
    char::from(match nibble {
        0..=9 => b'0' + nibble,
        _ => b'A' + nibble - 10,
    })
}

pub fn asked_for(url: &str, here: bool) -> Option<String> {
    if !here || !opens_the_web(url) {
        return None;
    }
    Some(format!("{SCHEME}://open?url={}", encoded(url.trim_start())))
}

pub fn opened_by<F>(url: &str, here: bool, mut open: F) -> bool
where
    F: FnMut(&str) -> bool,
{
    if let Some(through) = asked_for(url, here)
        && open(&through)
    {
        return true;
    }
    open(url)
}

#[cfg(test)]
#[path = "linkunbound_test.rs"]
mod tests;
