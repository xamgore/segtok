use std::sync::LazyLock;

use fancy_regex::Regex;

use super::{space_tokenizer, ALPHA_NUM};
use crate::regex::RegexSplitExt;

pub static SYMBOLIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r#"({ALPHA_NUM}+)"#)).unwrap());

/// The symbol tokenizer extends the [space_tokenizer] by separating alphanumerics.
///
/// Separates alphanumeric Unicode character sequences in already space-split tokens.
pub fn symbol_tokenizer(sentence: &str) -> impl Iterator<Item = &str> {
    space_tokenizer(sentence).flat_map(|token| SYMBOLIC.split_with_separators(token).filter(|&s| !s.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split() {
        let sentence = "  1a. --  http://www.ex_ample.com  ";
        let expected = ["1a", ".", "--", "http", "://", "www", ".", "ex", "_", "ample", ".", "com"];
        assert_eq!(symbol_tokenizer(sentence).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn unicode() {
        let sentence = "\u{0532}A\u{01CB}\u{0632}:\u{2580}%";
        let expected = ["\u{0532}A\u{01CB}", "\u{0632}:\u{2580}%"];
        assert_eq!(symbol_tokenizer(sentence).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn unicode_hyphens() {
        let sentence = "123-ABC\u{2011}DEF\u{2015}XYZ";
        let expected = ["123", "-", "ABC", "\u{2011}", "DEF", "\u{2015}", "XYZ"];
        assert_eq!(symbol_tokenizer(sentence).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn unicode_slashes() {
        let sentence = "kg/meter";
        let expected = ["kg", "/", "meter"];
        assert_eq!(symbol_tokenizer(sentence).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn superscript_numbers() {
        let sentence = "per m\u{00B3} earth"; // (superscript three)
        let expected = ["per", "m", "\u{00B3}", "earth"];
        assert_eq!(symbol_tokenizer(sentence).collect::<Vec<_>>(), expected);
    }
}
