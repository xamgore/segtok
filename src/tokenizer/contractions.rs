use std::sync::LazyLock;

use fancy_regex::Regex;

use super::{ALPHA_NUM, APOSTROPHES, HYPHEN, LIST_OF_APOSTROPHES};

/// A pattern that matches tokens with valid English contractions ``'(d|ll|m|re|s|t|ve)``.
pub static IS_CONTRACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r#"^{ALPHA_NUM}+(?:{HYPHEN}{ALPHA_NUM}+)*{APOSTROPHES}(?:d|ll|m|re|s|t|ve)$"#)).unwrap()
});

/// A function to split apostrophe contractions at the end of alphanumeric (and hyphenated) tokens.
///
/// Takes the output of a tokenizer function and produces an updated list.
///
/// **Note**: the original implementation contains a [bug](https://github.com/fnl/segtok/issues/26)
/// where multiple substrings were produced.
///
/// ```python
/// split_contractions(word_tokenizer("OʼHaraʼs"))
/// <<< ['OʼHara', 'O', 'ʼHaraʼs']
/// ```
pub fn split_contractions(mut tokens: Vec<String>) -> Vec<String> {
    let mut idx = 0;

    while idx < tokens.len() {
        let token = &mut tokens[idx];

        if token.len() > 1 && IS_CONTRACTION.is_match(token).unwrap() {
            if let Some((mut pos, ap)) = token.char_indices().rfind(|&(_, ch)| LIST_OF_APOSTROPHES.contains(ch)) {
                // don't, doesn't
                if token.get(pos.saturating_sub(1)..pos) == Some("n") && token.get(pos + ap.len_utf8()..) == Some("t") {
                    pos = pos.saturating_sub(1);
                }

                let suffix = token.split_off(pos);
                idx += 1;
                tokens.insert(idx, suffix);
            }
        }

        idx += 1;
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn misses() {
        assert!(!IS_CONTRACTION.is_match("don'r").unwrap());
        assert!(!IS_CONTRACTION.is_match("'ve").unwrap());
    }

    #[test]
    fn matches() {
        assert!(IS_CONTRACTION.is_match("I've").unwrap());
        assert!(IS_CONTRACTION.is_match("don't").unwrap());
    }

    #[test]
    fn unicode() {
        assert!(IS_CONTRACTION.is_match("Frank\u{02BC}s").unwrap());
        // assert!(IS_POSSESSIVE.is_match("Charles\u{2019}").unwrap());
        // assert!(IS_POSSESSIVE.is_match("home-less\u{2032}").unwrap());
    }

    #[test]
    fn split_regular() {
        let res = split_contractions(["We'll", "see", "her's", "too", "!"].map(ToOwned::to_owned).to_vec());
        assert_eq!(res.len(), 7);
        assert_eq!(res[0], "We");
        assert_eq!(res[1], "'ll");
        assert_eq!(res[3], "her");
        assert_eq!(res[4], "'s");
    }

    #[test]
    fn split_not() {
        let res = split_contractions(vec!["don't".to_owned()]);
        assert_eq!(res, ["do", "n't"]);
    }

    #[test]
    fn split_not_with_alternative_apostrophe() {
        let res = split_contractions(vec!["won’t".to_owned()]);
        assert_eq!(res, ["wo", "n’t"]);
    }

    #[test]
    fn split_unicode() {
        let res = split_contractions(vec!["a\u{2032}d".to_owned()]);
        assert_eq!(res, ["a", "\u{2032}d"]);
    }

    #[test]
    fn split_multiple() {
        // see: https://github.com/fnl/segtok/issues/26
        let res = split_contractions(vec!["OʼHaraʼs".to_owned()]);
        assert_eq!(res, ["OʼHara", "ʼs"]);
    }
}
