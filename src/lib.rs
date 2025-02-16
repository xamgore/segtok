use std::ops::Deref;

pub(crate) mod regex;
pub mod segmenter;
pub mod tokenizer;

/// Can be used in benchmarks.
#[doc(hidden)]
pub fn init() {
    let _ = segmenter::dates::MONTH.deref();
    let _ = segmenter::dates::ENDS_IN_DATE_DIGITS.deref();
    let _ = segmenter::BEFORE_LOWER.deref();
    let _ = segmenter::LOWER_WORD.deref();
    let _ = segmenter::MIDDLE_INITIAL_END.deref();
    let _ = segmenter::UPPER_WORD_START.deref();
    let _ = segmenter::LONE_WORD.deref();
    let _ = segmenter::UPPER_CASE_END.deref();
    let _ = segmenter::UPPER_CASE_START.deref();
    let _ = segmenter::DO_NOT_CROSS_LINES.deref();
    let _ = segmenter::MAY_CROSS_ONE_LINE.deref();
    let _ = segmenter::ABBREVIATIONS.deref();
    let _ = segmenter::CONTINUATIONS.deref();

    let _ = tokenizer::APOSTROPHE_LIKE.deref();
    let _ = tokenizer::HYPHENATED_LINEBREAK.deref();
    let _ = tokenizer::IS_CONTRACTION.deref();
    let _ = tokenizer::IS_POSSESSIVE.deref();
    let _ = tokenizer::SYMBOLIC.deref();
    let _ = tokenizer::URI_OR_MAIL.deref();
    let _ = tokenizer::WORD_BITS.deref();
}
