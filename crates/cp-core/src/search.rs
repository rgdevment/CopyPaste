use unicode_normalization::UnicodeNormalization;

pub fn fold(input: &str) -> String {
    input
        .nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(expand_ligature)
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_combining_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x20D0..=0x20FF)
}

fn expand_ligature(c: char) -> std::vec::IntoIter<char> {
    let expanded: Vec<char> = match c {
        'ß' => vec!['s', 's'],
        'æ' | 'Æ' => vec!['a', 'e'],
        'œ' | 'Œ' => vec!['o', 'e'],
        'ø' | 'Ø' => vec!['o'],
        'ł' | 'Ł' => vec!['l'],
        'đ' | 'Đ' => vec!['d'],
        'þ' | 'Þ' => vec!['t', 'h'],
        other => vec![other],
    };
    expanded.into_iter()
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn folds_both_sides_of_the_index() {
        assert_eq!(fold("Straße"), "strasse");
        assert_eq!(fold("encyclopædia"), "encyclopaedia");
        assert_eq!(fold("Łódź"), "lodz");
        assert_eq!(fold("el café"), "el cafe");
    }
}
