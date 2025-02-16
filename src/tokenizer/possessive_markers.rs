use std::sync::LazyLock;

use fancy_regex::Regex;
use itertools::Itertools;

use super::{is_apostrophe, ALPHA_NUM, APOSTROPHES, HYPHEN};

/// A pattern that matches English words with a possessive s terminal form.
pub static IS_POSSESSIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r#"^{ALPHA_NUM}+(?:{HYPHEN}{ALPHA_NUM}+)*(?:{APOSTROPHES}[sS]|[sS]{APOSTROPHES})$"#,)).unwrap()
});

/// A function to split possessive markers at the end of alphanumeric (and hyphenated) tokens.
///
/// Takes the output of any of the tokenizer functions and produces and updated list.
/// To use it, simply wrap the tokenizer function, for example::
///
/// ```ignored
/// assert_eq!(
///   split_possessive_markers(word_tokenizer("This is Fred's latest book.")),
///   ['This', 'is', 'Fred', "'s", 'latest', 'book', '.']
/// );
/// ```
pub fn split_possessive_markers(mut tokens: Vec<String>) -> Vec<String> {
    let mut idx = 0;

    while idx < tokens.len() {
        let token = &mut tokens[idx];

        if IS_POSSESSIVE.is_match(token).unwrap() {
            if let Some(((_2idx, _2ch), (_1idx, _1ch))) = token.char_indices().tuple_windows::<(_, _)>().last() {
                if _1ch.to_ascii_lowercase() == 's' && is_apostrophe(_2ch) {
                    let suffix = token.split_off(_2idx);
                    idx += 1;
                    tokens.insert(idx, suffix);
                } else if _2ch.to_ascii_lowercase() == 's' && is_apostrophe(_1ch) {
                    let suffix = token.split_off(_1idx);
                    idx += 1;
                    tokens.insert(idx, suffix);
                }
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
        assert!(!IS_POSSESSIVE.is_match("Frank'd").unwrap());
        assert!(!IS_POSSESSIVE.is_match("s'").unwrap());
    }

    #[test]
    fn matches() {
        assert!(IS_POSSESSIVE.is_match("Frank's").unwrap());
        assert!(IS_POSSESSIVE.is_match("Charles'").unwrap());
    }

    #[test]
    fn unicode() {
        assert!(IS_POSSESSIVE.is_match("Frank\u{02BC}s").unwrap());
        assert!(IS_POSSESSIVE.is_match("Charles\u{2019}").unwrap());
        assert!(IS_POSSESSIVE.is_match("home-less\u{2032}").unwrap());
    }

    #[test]
    fn split_with_s() {
        let res = split_possessive_markers(["Fred's", "is", "Frank's", "bar", "."].map(ToOwned::to_owned).to_vec());
        assert_eq!(res.len(), 7);
        assert_eq!(res[0], "Fred");
        assert_eq!(res[1], "'s");
        assert_eq!(res[3], "Frank");
        assert_eq!(res[4], "'s");
    }

    #[test]
    fn split_without_s() {
        let res = split_possessive_markers(vec!["CHARLES'".to_owned()]);
        assert_eq!(res, ["CHARLES", "'"]);
    }

    #[test]
    fn split_unicode() {
        assert!(is_apostrophe('\u{2032}'));
        let res = split_possessive_markers(vec!["a\u{2032}s".to_owned()]);
        assert_eq!(res, ["a", "\u{2032}s"]);
    }
}
