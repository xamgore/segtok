use std::sync::LazyLock;

use fancy_regex::{Captures, Regex};

use super::{
    is_non_quote_apostrophe, space_tokenizer, ALPHA_NUM, HYPHEN, HYPHENATED_LINEBREAK, LETTER, NON_QUOTE_APOSTROPHE,
    NUMBER,
};
use crate::regex::{Partition, PartitionIter};
use crate::segmenter::is_sentence_terminal;

pub static WORD_BITS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(?ux)
            (?:
              {ALPHA_NUM}
              (?:
                # Dots, except ellipsis
                \. (?! \.\. )
                # Comma, surrounded by digits (e.g., chemicals) or letters OR
                # ASCII single quote, surrounded by digits or letters (no dangling allowed)
              | [,'] (?={ALPHA_NUM})
                # Hyphen, surrounded by digits (e.g., DNA endings: "5'-ACGT-3'") or letters
                # incl. optional apostrophe for DNA segments
              | {NON_QUOTE_APOSTROPHE}? {HYPHEN} (?={ALPHA_NUM})
              )
            | # Colon, surrounded by digits (e.g., time, references)
              {NUMBER} : (?={NUMBER})
            | # Apostophes, non-consecutive
              {NON_QUOTE_APOSTROPHE} (?!{NON_QUOTE_APOSTROPHE})
            | # ASCII single quote after an s and at the token's end
              s ' $
            | # Terminal dimensions (superscript minus, 1, 2, and 3) attached to physical units
              #   size-prefix           unit-acronym     dimension
              \b [yzafpnµmcdhkMGTPEZY]? {LETTER}{{1,3}} ⁻?[¹²³] $
            | # Atom counts (subscript numbers) and ionization states (optional superscript
              # ² or ³ followed by a ⁺ or ⁻) are attached to valid fragments of a chemical formula
              \b (?: [A-Z][a-z]? | [\)\]] )+ [₀-₉]+ (?: [²³]?[⁺⁻] )?
            | # Any (Unicode) letter, digit, or the underscore
              {ALPHA_NUM}
            )+
        "#
    ))
    .unwrap()
});

/// This tokenizer extends the alphanumeric [symbol_tokenizer](crate::tokenizer::symbol_tokenizer)
/// by splitting fewer cases.
///
/// 1. Dots appearing after a letter are maintained as part of the word, except for the last word
///    in a sentence if that dot is the sentence terminal. Therefore, abbreviation marks (words
///    containing or ending in a ``.``, like "i.e.") remain intact and URL or ID segments remain
///    complete ("www.ex-ample.com", "EC1.2.3.4.5", etc.). The only dots that never are attached
///    are triple dots (``...``; ellipsis).
/// 2. Commas surrounded by alphanumeric characters are maintained in the word, too, e.g. ``a,b``.
///    Colons surrounded by digits are maintained, e.g., 'at 12:30pm' or 'Isaiah 12:3'.
///    Commas, semi-colons, and colons dangling at the end of a token are always spliced off.
/// 3. Any two alphanumeric letters that are separated by a single hyphen are joined together;
///    Those "inner" hyphens may optionally be followed by a linebreak surrounded by spaces;
///    The spaces will be removed, however. For example, ``Hel- \\r\\n \t lo`` contains a (Windows)
///    linebreak and will be returned as ``Hel-lo``.
/// 4. Apostrophes are always allowed in words as long as they are not repeated; The single quote
///    ASCII letter ``'`` is only allowed as a terminal apostrophe after the letter ``s``,
///    otherwise it must be surrounded by letters. To support DNA and chemicals, a apostrophe
///    (prime) may be located before the hyphen, as in the single token "5'-ACGT-3'" (if any
///    non-ASCII hyphens are used instead of the shown single quote).
/// 5. Superscript 1, 2, and 3, optionally prefixed with a superscript minus, are attached to a
///    word if it is no longer than 3 letters (optionally 4 if the first letter is a power prefix
///    in the range from yocto, y (10^-24) to yotta, Y (10^+24)).
/// 6. Subscript digits are attached if prefixed with letters that look like a chemical formula.
pub fn word_tokenizer(sentence: &str) -> Vec<String> {
    let pruned = HYPHENATED_LINEBREAK.replace_all(sentence, |caps: &Captures| format!("{}{}", &caps[1], &caps[2]));

    let (mut tokens, is_word_bit): (Vec<_>, Vec<_>) = space_tokenizer(&pruned)
        .flat_map(|span| PartitionIter::new(&WORD_BITS, span).filter(|&s| !s.as_ref().is_empty()))
        .map(Partition::into_pair)
        .unzip();

    // splice the sentence terminal off the last word/token if it has any at its borders
    // only look for the sentence terminal in the last three tokens
    let last_three = tokens.iter().copied().zip(is_word_bit.iter().copied()).enumerate().rev().take(3);

    for (idx, (word, is_word_bit)) in last_three {
        if is_word_bit && !word.chars().any(is_non_quote_apostrophe)
            || word.chars().last().is_some_and(is_sentence_terminal)
            || word.chars().next().is_some_and(is_sentence_terminal)
        {
            if word.chars().count() == 1 || word == "..." {
                break; // leave the token as it is
            }

            if let Some((pos, _)) = word.char_indices().last().filter(|&(_, last)| is_sentence_terminal(last)) {
                // stuff.
                let (prefix, suffix) = word.split_at(pos);
                tokens[idx] = prefix;
                tokens.insert(idx + 1, suffix);
            } else if let Some((pos, ch)) = word.char_indices().next().filter(|&(_, first)| is_sentence_terminal(first))
            {
                // .stuff
                let (prefix, suffix) = word.split_at(pos + ch.len_utf8());
                tokens[idx] = prefix;
                tokens.insert(idx + 1, suffix);
            }

            break;
        }
    }

    // keep splicing off any dangling commas and (semi-) colons
    for idx in (0..tokens.len()).rev() {
        let word = tokens[idx];
        if word.chars().count() <= 1 {
            continue;
        }
        if let Some((pos, _)) = word.char_indices().rev().take_while(|&(_, ch)| matches!(ch, ',' | ';' | ':')).last() {
            tokens.splice(
                idx..=idx,
                std::iter::once(&word[..pos]).chain(word[pos..].split("")).filter(|s| !s.is_empty()),
            );
        }
    }

    // we can't return reference the pruned string
    tokens.into_iter().map(ToOwned::to_owned).collect()
}

#[allow(clippy::needless_borrow)]
#[cfg(test)]
mod tests {
    use super::*;

    fn test_with(inner: char) {
        let input = format!(" 123{inner}456 abc{inner}def ");
        let expected = [format!("123{inner}456"), format!("abc{inner}def")];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn with_inner_hyphen() {
        test_with('-')
    }

    #[test]
    fn with_inner_comma() {
        test_with(',')
    }

    #[test]
    fn with_inner_dot() {
        test_with('.')
    }

    #[test]
    fn with_inner_colon() {
        let input = "12:6 12:50";
        let expected = ["12:6", "12:50"];
        assert_eq!(word_tokenizer(&input), expected);

        let input = "abc:def 12:34:abc abc:12:34";
        let expected = ["abc", ":", "def", "12:34", ":", "abc", "abc", ":", "12:34"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    fn test_dangling(char: char) {
        let input = format!("that {char}but not{char} this");
        let expected = ["that", &char.to_string(), "but", "not", &char.to_string(), "this"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn with_dangling_hyphen() {
        test_dangling('-')
    }

    #[test]
    fn with_dangling_comma() {
        test_dangling(',')
    }

    #[test]
    fn with_dangling_colon() {
        test_dangling(':')
    }

    #[test]
    fn with_dangling_semicolon() {
        test_dangling(';')
    }

    #[test]
    fn dangling_comma_twice() {
        let input = "token (, hi), issue";
        let expected = ["token", "(", ",", "hi", ")", ",", "issue"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn dangling_comma_dangling_double() {
        let input = "token (,; hi), issue";
        let expected = ["token", "(", ",", ";", "hi", ")", ",", "issue"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    fn test_terminal(char: char) {
        let input = format!("A{char}");
        let expected = ["A", &char.to_string()];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn with_terminal_hyphen() {
        test_terminal('-')
    }

    #[test]
    fn with_terminal_comma() {
        test_terminal(',')
    }

    #[test]
    fn with_terminal_colon() {
        test_terminal(':')
    }

    #[test]
    fn with_terminal_semicolon() {
        test_terminal(';')
    }

    #[test]
    fn hyphen_repeat() {
        let input = "A--B";
        let expected = ["A", "--", "B"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn comma_repeat() {
        let input = "A,,B";
        let expected = ["A", ",", ",", "B"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn hyphen_unicode() {
        let input = "\u{00A0}ABC\u{2011}DEF\u{2015}XYZ\u{00A0}";
        let expected = ["ABC\u{2011}DEF", "\u{2015}", "XYZ"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn hyphen_mixed() {
        let input = "123-Abc-xyZ-123";
        let expected = ["123-Abc-xyZ-123"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn hyphen_linebreak() {
        let input = "A-B A-\rB A-\nB A-  \r\n\tB";
        let expected = ["A-B", "A-B", "A-B", "A-B"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn dots() {
        let input = "\t1.2.3, f.e., is Mr. .Abbreviation.\n";
        let expected = ["1.2.3", ",", "f.e.", ",", "is", "Mr.", ".", "Abbreviation", "."];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn hyphened_numbers() {
        let input = "1-1-1:2:2";
        let expected = ["1-1-1:2:2"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn splice_sentence_terminal_start() {
        let input = "This is a ?sentence,";
        let expected = ["This", "is", "a", "?", "sentence", ","];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn splice_sentence_terminal_end() {
        let input = "This is a sentence?,";
        let expected = ["This", "is", "a", "sentence", "?", ","];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn final_abbreviation() {
        let input = "This is another abbrev..\n";
        let expected = ["This", "is", "another", "abbrev.", "."];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn final_ellipsis() {
        let input = "Please no more...";
        let expected = ["Please", "no", "more", "..."];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn abbreviated_ellipsis() {
        let input = "abbrev... final....";
        let expected = ["abbrev", "...", "final", "...", "."];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn double_dot() {
        let input = "a.. b..";
        let expected = ["a.", ".", "b.", "."];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn dot_apo_single_quote() {
        let input = "He said, 'this.'";
        let expected = ["He", "said", ",", "'", "this", ".", "'"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn ellipsis_inner() {
        let input = "and...or";
        let expected = ["and", "...", "or"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn apostrophe_simple() {
        // NB: ASCII single quote "apostrophe" (ab-) use is to unsafe to maintain attached...
        let input = "That's 'tis less' O'Don'Ovan's";
        let expected = ["That's", "'", "tis", "less'", "O'Don'Ovan's"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn possesive_s_ascii_apostrophe() {
        // NB: ...except for the clear case of "...s'"
        let input = "Words' end.";
        let expected = ["Words'", "end", "."];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn apostrophe_unicode() {
        let input = "\u{2019}tis less\u{02BC} O\u{2019}Neil\u{02BC}s";
        let expected = ["\u{2019}tis", "less\u{02BC}", "O\u{2019}Neil\u{02BC}s"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn hyphen_dot_apostrophe() {
        let input = " O.h'Ne.l- \n l's ";
        let expected = ["O.h'Ne.l-l's"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn numbers() {
        let input = "$123,456.99 45.67+/-1.23%";
        let expected = ["$", "123,456.99", "45.67", "+/-", "1.23", "%"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn chemicals_and_dna() {
        let input = "1,r-4-cyclo.hexene 5′-ATGCAAAT-3′ 5'-ACGT-3'";
        // this one is too ambiguous
        let expected = ["1,r-4-cyclo.hexene", "5′-ATGCAAAT-3′", "5", "'-", "ACGT-3", "'"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn physical_units() {
        let input = "10 V·m⁻¹ msec²";
        let expected = ["10", "V", "·", "m⁻¹", "msec²"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn chemical_formula() {
        let input = "O₂ H₁₂Si₅O₂ Al₂(SO₄)₃ [NO₄]⁻ Not₁";
        let expected = ["O₂", "H₁₂Si₅O₂", "Al₂", "(", "SO₄", ")₃", "[", "NO₄", "]⁻", "Not", "₁"];
        assert_eq!(word_tokenizer(&input), expected);
    }

    #[test]
    fn urls() {
        let input = "http://www.example.com/path/to.file?kwd=1&arg";
        let expected =
            ["http", "://", "www.example.com", "/", "path", "/", "to.file", "?", "kwd", "=", "1", "&", "arg"];
        assert_eq!(word_tokenizer(&input), expected);
    }
}
