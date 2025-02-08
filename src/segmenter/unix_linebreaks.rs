use std::borrow::Cow;
use std::sync::LazyLock;

use fancy_regex::Regex;

/// All linebreak sequence variants except the Unix newline (only).
#[deprecated]
pub static NON_UNIX_LINEBREAK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\r\n|\r|\u{2028}"#).unwrap());

/// Replace non-Unix linebreak sequences (Windows, Mac, Unicode) with newlines (`\n`).
#[deprecated]
#[allow(deprecated)]
pub fn to_unix_linebreaks(text: &str) -> Cow<str> {
    NON_UNIX_LINEBREAK.replace_all(text, "\n")
}

#[allow(deprecated)]
#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let result = to_unix_linebreaks("This\r\none.");
        assert_eq!(result, "This\none.");
    }

    #[test]
    fn test_NON_UNIX_LINEBREAK_search() {
        for example in ["\r", "\r\n", "\u{2028}"] {
            assert!(NON_UNIX_LINEBREAK.is_match(example).unwrap());
        }
    }

    #[test]
    fn test_NON_UNIX_LINEBREAK_misses() {
        for example in ["\n", " ", "\t"] {
            assert!(!NON_UNIX_LINEBREAK.is_match(example).unwrap());
        }
    }
}
