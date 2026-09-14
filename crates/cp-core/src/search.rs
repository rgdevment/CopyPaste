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
    fn every_ligature_in_the_table_is_expanded() {
        for (given, expected) in [
            ("ß", "ss"),
            ("æ", "ae"),
            ("Æ", "ae"),
            ("œ", "oe"),
            ("Œ", "oe"),
            ("ø", "o"),
            ("Ø", "o"),
            ("ł", "l"),
            ("Ł", "l"),
            ("đ", "d"),
            ("Đ", "d"),
            ("þ", "th"),
            ("Þ", "th"),
        ] {
            assert_eq!(fold(given), expected, "«{given}» no se expandió bien");
        }
    }

    #[test]
    fn combining_marks_disappear_whatever_the_script() {
        assert_eq!(fold("ñ"), "n");
        assert_eq!(fold("ç"), "c");
        assert_eq!(fold("ü"), "u");
        assert_eq!(fold("å"), "a");
        assert_eq!(fold("ĉ"), "c");
        assert_eq!(fold("ṩ"), "s");
    }

    #[test]
    fn what_has_nothing_to_fold_comes_out_as_it_went_in() {
        assert_eq!(fold(""), "");
        assert_eq!(fold("plain ascii"), "plain ascii");
        assert_eq!(fold("日本語"), "日本語");
        assert_eq!(fold("123 !?"), "123 !?");
    }

    #[test]
    fn folding_twice_changes_nothing() {
        for text in ["Straße", "encyclopædia", "Łódź", "café", "ÞÓRR"] {
            let once = fold(text);
            assert_eq!(fold(&once), once, "«{text}» no era estable");
        }
    }

    #[test]
    fn folds_both_sides_of_the_index() {
        assert_eq!(fold("Straße"), "strasse");
        assert_eq!(fold("encyclopædia"), "encyclopaedia");
        assert_eq!(fold("Łódź"), "lodz");
        assert_eq!(fold("el café"), "el cafe");
    }
}
