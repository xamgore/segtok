use std::sync::LazyLock;

use fancy_regex::Regex;

// PMC OA corpus statistics
// SSs: sentence starters
// abbrevs: abbreviations
//
// Words likely used as SSs (poor continuations, >10%):
// after, though, upon, while, yet
//
// Words hardly used after abbrevs vs. SSs (poor continuations, <2%):
// [after], as, at, but, during, for, in, nor, on, to, [though], [upon],
// whereas, [while], within, [yet]
//
// Words hardly ever used as SSs (excellent continuations, <2%):
// and, are, between, by, from, has, into, is, of, or, that, than, through,
// via, was, were, with
//
// Words frequently used after abbrevs (excellent continuations, >10%):
// [and, are, has, into, is, of, or, than, via, was, were]
//
// Grey zone: undecidable words -> leave in to bias towards under-splitting
// whether

/// Lower-case words that in the given form usually don't start a sentence.
pub static CONTINUATIONS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)
            ^ # at string start only
            (?: a(?: nd|re )
            |   b(?: etween|y )
            |   from
            |   has
            |   i(?: nto|s )
            |   o[fr]
            |   t(?: han|hat|hrough )
            |   via
            |   w(?: as|ere|hether|ith )
            )\b
        "#,
    )
    .unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detected() {
        for example in ["and this", "are those"] {
            assert!(CONTINUATIONS.is_match(example).unwrap());
        }
    }

    #[test]
    fn ignored() {
        for example in ["to be", "Are those", "not and"] {
            assert!(!CONTINUATIONS.is_match(example).unwrap());
        }
    }
}
