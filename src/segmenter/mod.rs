//! A pattern-based sentence segmentation strategy.
//!
//! Known limitations:
//!
//! 1. The sentence must use a known sentence terminal followed by space(s),
//!    skipping one optional, intervening quote and/or bracket.
//! 2. The next sentence must start with an upper-case letter or a number,
//!    ignoring one optional quote and/or bracket before it.
//!    Alternatively, it may start with a camel-cased word, like "gene-A".
//! 3. If the sentence ends with a single upper-case letter followed by a dot,
//!    a split is made (splits names like "A. Dent"), unless there is an easy
//!    to deduce reason that it is a human name.
//!
//! The decision for requiring an "syntactically correct" terminal sequence with upper-case letters or
//! numbers as start symbol is based on the preference to under-split rather than over-split sentences.
//!
//! Special care is taken not to split at common abbreviations like "i.e." or "etc.",
//! to not split at first or middle name initials "... F. M. Last ...",
//! to not split before a comma, colon, or semi-colon,
//! and to avoid single letters or digits as sentences ("A. This sentence...").
//!
//! Sentence splits will always be enforced at **consecutive** line separators.
//!
//! Important: Windows text files use `\r\n` as linebreaks and Mac files use `\r`;
//! Convert the text to Unix linebreaks if the case.

mod abbreviations;
mod continuations;
mod unix_linebreaks;

use std::cmp::Ordering;
use std::sync::LazyLock;

use fancy_regex::Regex;

pub use self::abbreviations::*;
pub use self::continuations::*;
pub use self::dates::*;
pub use self::unix_linebreaks::*;
use super::regex::RegexSplitExt;

pub mod dates {
    //! Special facilities to detect European-style dates.

    use super::*;

    pub static ENDS_IN_DATE_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\b[0123]?[0-9]$"#).unwrap());

    pub static MONTH: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(J[äa]n|Ene|Feb|M[äa]r|A[pb]r|May|Jun|Jul|Aug|Sep|O[ck]t|Nov|D[ei][cz]|0?[1-9]|1[012])").unwrap()
    });
}

/// Any valid word-breaking hyphen, including ASCII hyphen minus.
pub const HYPHENS: &str = r#"\u{00AD}\u{058A}\u{05BE}\u{0F0C}\u{1400}\u{1806}\u{2010}-\u{2012}\u{2e17}\u{30A0}-"#;

/// The list of valid Unicode sentence terminal characters.
pub const SENTENCE_TERMINALS: &str =
    r#".!?\u{203C}\u{203D}\u{2047}\u{2048}\u{2049}\u{3002}\u{FE52}\u{FE57}\u{FF01}\u{FF0E}\u{FF1F}\u{FF61}"#;

#[deprecated]
pub const LIST_OF_SENTENCE_TERMINALS: &str =
    ".!?\u{203C}\u{203D}\u{2047}\u{2048}\u{2049}\u{3002}\u{FE52}\u{FE57}\u{FF01}\u{FF0E}\u{FF1F}\u{FF61}";

#[inline]
pub(crate) fn is_sentence_terminal(ch: char) -> bool {
    matches!(
        ch,
        '.' | '!'
            | '?'
            | '\u{203C}'
            | '\u{203D}'
            | '\u{2047}'
            | '\u{2048}'
            | '\u{2049}'
            | '\u{3002}'
            | '\u{FE52}'
            | '\u{FE57}'
            | '\u{FF01}'
            | '\u{FF0E}'
            | '\u{FF1F}'
            | '\u{FF61}'
    )
}

/// Endings that, if followed by a lower-case word, are not sentence terminals:
/// - quotations and brackets ("Hello!" said the man.)
/// - dotted abbreviations (U.S.A. was)
/// - genus-species-like (m. musculus)
pub static BEFORE_LOWER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(?uxs)
            (?:
              [{SENTENCE_TERMINALS}] (?: " [)\]]* | [)\]]+ )   # ."]) .") ."  OR  .])  .)
            | \b (?: spp | \p{{L}} \p{{Ll}}? ) \.              # spp.  (species pluralis)  OR  Ll. L.
            )
            \s+ $
        "#
    ))
    .unwrap()
});

/// Lower-case words are not sentence starters (after an abbreviation).
pub static LOWER_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r#"^\p{{Ll}}+[{HYPHENS}]?\p{{Ll}}*\b"#)).unwrap());

/// Upper-case initial after upper-case word at the end of a string.
pub static MIDDLE_INITIAL_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\b\p{Lu}\p{Ll}+\W+\p{Lu}$"#).unwrap());

/// Upper-case word at the beginning of a string.
pub static UPPER_WORD_START: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\p{Lu}\p{Ll}+\b"#).unwrap());

/// Any 'lone' lower-case word **with hyphens or digits inside** is a continuation.
pub static LONE_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r#"^\p{{Ll}}+[\p{{Ll}}\p{{Nd}}{HYPHENS}]*$"#)).unwrap());

/// Inside brackets, 'Words' that can be part of a proper noun abbreviation, like a journal name.
pub static UPPER_CASE_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\b[\p{Lu}\p{Lt}]\p{L}*\.\s+$"#).unwrap());

/// Inside brackets, 'Words' that can be part of a large abbreviation, like a journal name.
pub static UPPER_CASE_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(?:(?:\(\d{4}\)\s)?[\p{Lu}\p{Lt}]\p{L}*|\d+)[\.,:]\s+"#).unwrap());

/// Sentence end a sentence terminal, followed by spaces.
/// Optionally, a right quote and any number of closing brackets may succeed the terminal marker.
/// Alternatively, a yet undefined number of line-breaks also may terminate sentences.
fn segmenter_regex(line_breaks: usize) -> Regex {
    Regex::new(&format!(
        r#"(?ux)
            (                               # A sentence ends at one of two sequences:
                [{SENTENCE_TERMINALS}]      # Either, a sequence starting with a sentence terminal,
                ['’"”]?                     #         an optional right quote,
                [\]\)]*                     #         optional closing brackets and
                \s+                         #         a sequence of required spaces.
            |                               # Otherwise,
                \n{{{line_breaks},}}        #         a sentence also terminates at [consecutive] newlines.
            )
        "#
    ))
    .unwrap()
}

/// A segmentation pattern where any newline char also terminates a sentence.
pub static DO_NOT_CROSS_LINES: LazyLock<Regex> = LazyLock::new(|| segmenter_regex(1));

/// A segmentation pattern where two or more newline chars also terminate sentences.
pub static MAY_CROSS_ONE_LINE: LazyLock<Regex> = LazyLock::new(|| segmenter_regex(2));

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SegmentConfig {
    join_on_lowercase: bool,
    /// Length of either sentence fragment inside brackets to assume the fragment is not its own sentence.
    ///
    /// This can be increased/decreased to heighten/lower the likelihood of splits inside brackets.
    short_sentence_length: usize,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self { join_on_lowercase: false, short_sentence_length: 55 }
    }
}

/// Default: split `text` at sentence terminals and at newline chars.
pub fn split_single(text: &str, cfg: SegmentConfig) -> Vec<String> {
    let sentences = sentences(DO_NOT_CROSS_LINES.split_with_separators(text), cfg);
    sentences.iter().flat_map(|sentence| sentence.split("\n").map(ToOwned::to_owned)).collect()
}

/// Sentences may contain non-consecutive (single) newline chars,
/// while consecutive newline chars ("paragraph separators") always split sentences.
pub fn split_multi(text: &str, cfg: SegmentConfig) -> Vec<String> {
    sentences(MAY_CROSS_ONE_LINE.split_with_separators(text), cfg)
}

/// Split the `text` at newlines (``\\n'') and strip the lines,
/// but only return lines with content.
pub fn split_newline(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n').map(str::trim).filter(|&s| !s.is_empty())
}

/// Join spans back together into sentences as necessary.
fn sentences<'a>(spans: impl Iterator<Item = &'a str>, cfg: SegmentConfig) -> Vec<String> {
    let shorter_than_a_typical_sentence = |x: usize, y: usize| x.min(y) < cfg.short_sentence_length;

    let mut _last: Option<String> = None;
    let spans = spans.collect::<Vec<_>>();
    let mut res = Vec::with_capacity(spans.len());

    for current in join_abbreviations(&spans) {
        match _last {
            None => {
                _last = Some(current);
            }
            Some(ref mut last) => {
                if (cfg.join_on_lowercase || BEFORE_LOWER.is_match(last).unwrap())
                    && LOWER_WORD.is_match(&current).unwrap()
                    || (shorter_than_a_typical_sentence(current.len(), last.len())
                        && (is_open(last, ('(', ')'))
                            && (is_not_open(&current, ('(', ')'))
                                || last.ends_with(" et al. ")
                                || (UPPER_CASE_END.is_match(last).unwrap()
                                    && UPPER_CASE_START.is_match(&current).unwrap())))
                        || (is_open(last, ('[', ']'))
                            && (is_not_open(&current, ('[', ']'))
                                || last.ends_with(" et al. ")
                                || (UPPER_CASE_END.is_match(last).unwrap()
                                    && UPPER_CASE_START.is_match(&current).unwrap()))))
                    || CONTINUATIONS.is_match(&current).unwrap()
                {
                    last.push_str(&current)
                } else {
                    res.push(last.trim().to_string());
                    _last = Some(current);
                }
            }
        }
    }

    _last.inspect(|last| res.push(last.trim().to_string()));
    res
}

/// Join spans that match the `ABBREVIATIONS` pattern.
fn join_abbreviations(spans: &[&str]) -> Vec<String> {
    let mut res = Vec::with_capacity(spans.len());
    let mut put = |start, end| res.push(spans[start..end].join(""));

    fn ends_with_whitespace(str: &str) -> bool {
        str.bytes().next_back().is_some_and(|ch| ch.is_ascii_whitespace())
    }

    let mut from = None;
    for pos in 0..spans.len() {
        if pos % 2 == 0 {
            from = from.or(Some(pos));
        } else {
            let prev = spans[pos - 1];
            let marker = spans[pos];
            let next = spans.get(pos + 1);

            if ends_with_whitespace(prev)
                || marker.starts_with('.') && (ABBREVIATIONS.is_match(prev).unwrap())
                || next.is_some_and(|&next| {
                    LONE_WORD.is_match(next).unwrap()
                        || (ENDS_IN_DATE_DIGITS.is_match(prev).unwrap() && MONTH.is_match(next).unwrap())
                        || (MIDDLE_INITIAL_END.is_match(prev).unwrap() && UPPER_WORD_START.is_match(next).unwrap())
                })
            {
                continue;
            } else {
                from.inspect(|&from| put(from, pos + 1));
                from = None;
            }
        }
    }

    from.inspect(|&from| put(from, spans.len()));
    res
}

/// Check if the span ends with an unclosed ASCII `bracket`.
fn is_open(span: &str, brackets: (char, char)) -> bool {
    let mut offset = span.find(brackets.0);
    let mut nesting = if offset.is_none() { 0 } else { 1 };

    while let Some(idx) = offset {
        let idx = idx + 1;
        let opener = span[idx..].find(brackets.0).map(|i| i + idx);
        let closer = span[idx..].find(brackets.1).map(|i| i + idx);

        match (opener, closer) {
            (None, None) => {
                offset = None;
            }
            (None, Some(_)) => {
                offset = closer;
                nesting -= 1;
            }
            (Some(_), None) => {
                offset = opener;
                nesting += 1;
            }
            (Some(op), Some(cl)) => match op.cmp(&cl) {
                Ordering::Less => {
                    offset = opener;
                    nesting += 1;
                }
                Ordering::Greater => {
                    offset = closer;
                    nesting -= 1;
                }
                Ordering::Equal => {
                    unreachable!("open and closer have the same position")
                }
            },
        }
    }

    nesting > 0
}

/// Check if the span starts with an unopened ASCII `bracket`.
fn is_not_open(span: &str, brackets: (char, char)) -> bool {
    let mut offset = span.rfind(brackets.1);
    let mut nesting = if offset.is_none() { 0 } else { 1 };

    while let Some(idx) = offset {
        let opener = span[0..idx].rfind(brackets.0);
        let closer = span[0..idx].rfind(brackets.1);

        match (opener, closer) {
            (None, None) => {
                offset = None;
            }
            (None, Some(_)) => {
                offset = closer;
                nesting += 1;
            }
            (Some(_), None) => {
                offset = opener;
                nesting -= 1;
            }
            (Some(op), Some(cl)) => match op.cmp(&cl) {
                Ordering::Less => {
                    offset = closer;
                    nesting += 1;
                }
                Ordering::Greater => {
                    offset = opener;
                    nesting -= 1;
                }
                Ordering::Equal => {
                    unreachable!("open and closer have the same position")
                }
            },
        }
    }

    nesting > 0
}

#[cfg(test)]
mod tests {
    #![deny(unused_variables)]
    use std::sync::LazyLock;

    use super::*;

    const OSPL: &str = include_str!("ospl.txt");

    static SENTENCES: LazyLock<Vec<&'static str>> = LazyLock::new(|| OSPL.split("\n").collect::<Vec<_>>());

    static TEXT: LazyLock<String> = LazyLock::new(|| SENTENCES.join("\n"));

    #[test]
    fn try_newline() {
        assert_eq!(*SENTENCES, split_newline(OSPL).collect::<Vec<_>>())
    }

    #[test]
    fn try_regex() {
        let actual = split_single(&TEXT, Default::default());
        assert_eq!(actual, *SENTENCES);
    }

    fn test_split_single<const N: usize>(sentences: [&str; N]) {
        let text = sentences.join(" ");
        let expected: Vec<&str> = sentences.to_vec();
        let actual = split_single(&text, Default::default());
        assert_eq!(expected, actual);
    }

    #[test]
    fn try_simple() {
        test_split_single(["This is a test."])
    }

    #[test]
    fn try_names() {
        test_split_single([
            "Written by A. McArthur, K. Elvin, and D. Eden.",
            "This is Mr. A. Starr over there.",
            "B. Boyden is over there.",
        ])
    }

    #[test]
    fn try_alpha_items() {
        test_split_single(["This is figure A, B, and C.", "This is table A and B.", "That is item A, B."])
    }

    #[test]
    fn try_author_list() {
        test_split_single(["R. S. Kauffman, R. Ahmed, and B. N. Fields show stuff in their paper."])
    }

    #[test]
    fn try_long_bracket_abbreviation() {
        test_split_single([
      "This is expected, on the basis of (Olmsted, M. C., C. F. Anderson, and M. T. Record, Jr. 1989. Proc. Natl. Acad. Sci. USA. 100:100), to decrease sharply.",
    ])
    }

    #[test]
    fn try_continuations() {
        test_split_single([
            "colonic colonization inhibits development of inflammatory lesions.",
            "to investigate whether an inf. of the pancreas was the case...",
            "though we hate to use capital lett. that usually separate sentences.",
        ])
    }

    #[test]
    fn try_inner_names() {
        test_split_single([
            "Bla bla [Sim et al. (1981) Biochem. J. 193, 129-141].",
            "The adjusted (ml. min-1. 1.73 m-2) rate.",
        ])
    }

    #[test]
    fn try_species_names() {
        test_split_single([
            "Their presence was detected by transformation into S. lividans.",
            "Three subjects diagnosed as having something.",
        ])
    }

    #[test]
    fn try_species_names_tough() {
        test_split_single([
            "The level of the genus Allomonas gen. nov. with so far the only species A. enterica known.",
        ])
    }

    #[test]
    fn try_european_dates() {
        test_split_single([
            "Der Unfall am 24. Dezember 2016.",
            "Am 13. Jän. 2006 war es regnerisch.",
            "Am 13. 1. 2006 war es regnerisch.",
        ])
    }

    #[test]
    fn try_middle_name_initials() {
        test_split_single([
            "The administrative basis for Lester B. Pearson's foreign policy was developed later.",
            "This model was introduced by Dr. Edgar F. Codd after initial criticisms.",
        ])
    }

    #[test]
    fn try_parenthesis() {
        test_split_single([
            "Nested ((Parenthesis. (With words right (inside))) (More stuff. Uff, this is it!))",
            "In the Big City.",
        ])
    }

    #[test]
    fn try_parenthesis_with_sentences() {
        test_split_single([
            "The segmenter segments on single lines or to consecutive lines.",
            "(If you want to extract sentences that cross newlines, remove those line-breaks.",
            "Segtok assumes your content has some minimal semantical meaning.)",
            "It gracefully handles this and similar issues.",
        ])
    }

    #[test]
    fn try_unclosed_brackets() {
        test_split_single([
            "The medial preoptic area (MPOA), and 2) did not decrease Fos-lir.",
            "However, olfactory desensitizations did decrease Fos-lir.",
        ])
    }

    #[test]
    fn try_multiline() {
        let text = "This is a\nmultiline sentence. And this is Mr.\nAbbrevation.";
        let expected = ["This is a\nmultiline sentence.", "And this is Mr.\nAbbrevation."];
        let actual = split_multi(text, Default::default());
        assert_eq!(actual, expected);
    }

    #[test]
    fn try_linebreak() {
        let text = "This is a\nmultiline sentence.";
        let expected: Vec<&str> = text.split("\n").collect();
        let actual = split_single(text, Default::default());
        assert_eq!(actual, expected);
    }

    #[test]
    fn try_linebreak2() {
        let text = "Folding Beijing\nby Hao Jingfang";
        let expected: Vec<&str> = text.split("\n").collect();
        let actual = split_single(text, Default::default());
        assert_eq!(actual, expected);
    }
}
