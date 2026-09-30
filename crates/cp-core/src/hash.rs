use xxhash_rust::xxh3::xxh3_64;

const WHOLE_UP_TO: usize = 256 * 1024;
const BLOCKS: usize = 16;
const BLOCK: usize = 4 * 1024;

const _: () = assert!(WHOLE_UP_TO >= BLOCKS * BLOCK);
const _: () = assert!(BLOCK.is_power_of_two() && WHOLE_UP_TO.is_power_of_two());

pub fn content_hash(bytes: &[u8]) -> u64 {
    if bytes.len() <= WHOLE_UP_TO {
        return xxh3_64(bytes);
    }
    let mut mixed = Vec::with_capacity(BLOCKS * BLOCK + 8);
    mixed.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    for block in 0..BLOCKS {
        let start = bytes.len().saturating_sub(BLOCK) * block / (BLOCKS - 1);
        let end = (start + BLOCK).min(bytes.len());
        mixed.extend_from_slice(&bytes[start..end]);
    }
    xxh3_64(&mixed)
}

#[cfg(test)]
#[path = "hash_test.rs"]
mod tests;
