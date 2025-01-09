mod contractions;
mod possessive_markers;
mod space_tokenizer;
mod symbol_tokenizer;
mod web_tokenizer;
mod word_tokenizer;

use std::sync::LazyLock;

use fancy_regex::Regex;

pub use self::contractions::*;
pub use self::possessive_markers::*;
pub use self::space_tokenizer::*;
pub use self::symbol_tokenizer::*;
pub use self::web_tokenizer::*;
pub use self::word_tokenizer::*;

/// All apostrophe-like marks, including the ASCII "single quote".
pub const LIST_OF_APOSTROPHES: &str = "'\u{00B4}\u{02B9}\u{02BC}\u{2019}\u{2032}";

/// Any apostrophe-like marks, including "prime" but not the ASCII "single quote".
pub const APOSTROPHES: &str = r#"['\u{00B4}\u{02B9}\u{02BC}\u{2019}\u{2032}]"#;

/// Matcher for any apostrophe-like marks, including "prime" but not the ASCII "single quote".
pub static APOSTROPHE_LIKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[\u{00B4}\u{02B9}\u{02BC}\u{2019}\u{2032}]"#).unwrap());

/// Any valid linebreak sequence (Windows, Unix, Mac, or U+2028).
pub const LINEBREAK: &str = r#"(?:\r\n|\n|\r|\u{2028})"#;

/// Any Unicode letter character that can form part of a word: Ll, Lm, Lt, Lu.
pub const LETTER: &str = r#"[\p{Ll}\p{Lm}\p{Lt}\p{Lu}]"#;

/// Any Unicode number character: Nd or Nl.
pub const NUMBER: &str = r#"[\p{Nd}\p{Nl}]"#;

/// Any alphanumeric Unicode character: letter or number.
pub const ALPHA_NUM: &str = r#"[\p{Ll}\p{Lm}\p{Lt}\p{Lu}\p{Nd}\p{Nl}]"#;

/// Superscript 1, 2, and 3, optionally prefixed with a minus sign.
pub const POWER: &str = r#"\u{207B}?[\u{00B9}\u{00B2}\u{00B3}]"#;

/// Subscript digits.
pub const SUBDIGIT: &str = r#"[\u{2080}-\u{2089}]"#;

pub const HYPHEN: &str = r#"[\u{00AD}\u{058A}\u{05BE}\u{0F0C}\u{1400}\u{1806}\u{2010}-\u{2012}\u{2e17}\u{30A0}-]"#;

/// Any Unicode space character plus the (horizontal) tab.
pub const SPACE: &str = r#"[\p{Zs}\t]"#;

/// The pattern matches any alphanumeric Unicode character, followed by a hyphen,
/// A single line-break surrounded by optional (non-breaking) spaces,
/// and terminates with a alphanumeric character on this next line.
/// The opening char and hyphen as well as the terminating char are captured in two groups.
pub static HYPHENATED_LINEBREAK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r#"({ALPHA_NUM}{HYPHEN}){SPACE}*?{LINEBREAK}{SPACE}*?({ALPHA_NUM})"#)).unwrap()
});
