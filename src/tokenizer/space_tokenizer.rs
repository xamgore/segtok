use std::sync::LazyLock;

use fancy_regex::Regex;

static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\s+"#).unwrap());

/// For a given input `sentence`, return a list of its tokens.
///
/// Split on Unicode spaces ``\s+`` (i.e., any kind of **Unicode** space character).
/// The separating space characters are not included in the resulting token list.
pub fn space_tokenizer(sentence: &str) -> impl Iterator<Item = &str> {
    REGEX.split(sentence).map(Result::unwrap).filter(|&s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split() {
        let sentence = " 1\n2\t3  4\t\n 5 ";
        let expected = ["1", "2", "3", "4", "5"];
        assert_eq!(space_tokenizer(sentence).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn unicode() {
        let sentence = "1\u{00A0}2\u{2007} 3  \u{2007}  ";
        let expected = ["1", "2", "3"];
        assert_eq!(space_tokenizer(sentence).collect::<Vec<_>>(), expected);
    }
}
