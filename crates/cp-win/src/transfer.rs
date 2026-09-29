pub(crate) const COPY: u32 = 1;
const MOVE: u32 = 2;
const LINK: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transfer {
    Copy,
    Move,
    Unsaid,
}

pub fn transfer(value: &[u8]) -> Transfer {
    let [a, b, c, d, ..] = value else {
        return Transfer::Unsaid;
    };
    let effect = u32::from_le_bytes([*a, *b, *c, *d]);
    if effect & MOVE != 0 {
        return Transfer::Move;
    }
    if effect & COPY != 0 || effect & LINK != 0 {
        return Transfer::Copy;
    }
    Transfer::Unsaid
}

#[cfg(test)]
#[path = "transfer_test.rs"]
mod tests;
