use std::sync::LazyLock;

use fancy_regex::Regex;

use crate::iter_tools::join;
use crate::segmenter::HYPHENS;

/// Common abbreviations at the candidate sentence end that normally don't terminate a sentence.
/// Note that a check is required to ensure the potential abbreviation is actually followed
/// by a dot and not some other sentence segmentation marker.
pub static ABBREVIATIONS: LazyLock<Regex> = LazyLock::new(|| {
    // Only abbreviations that should never occur at the end of a sentence (such as "etc.")
    let list = r#"
        [Aa]pprox Capt [Cc]f Col Dr [Ff]\.?e [Ff]igs? Gen [Ee]\.?g [Ii]\.?e [Ii]\.?v
        Mag [Mm]ed Mr Mrs Mt [Nn]at No [Nn]r [Pp]\.e [Pp]hil [Pp]rof [Rr]er
        [Ss]ci Sgt Sr Sra Srta St [Uu]niv [Vv]ol [Vv]s [Zz]\.B
        Jän Jan Ene Feb Mär Mar Apr Abr May Jun Jul Aug Sep Sept Oct Okt Nov Dic Dez Dec
        E\.U U\.K U\.S
    "#;
    let list = join(list.split_ascii_whitespace(), "|");
    Regex::new(&format!(
        r#"(?ux)
        (?: \b(?:{list}) # 1. known abbreviations,
        |   ^\S          # 2. a single, non-space character "sentence" (only),
        |   ^\d+         # 3. a series of digits "sentence" (only), or
        |   (?: \b       # 4. terminal letters A.-A, A.A, or A, if prefixed with:
            # 4.a. something that makes them most likely a human first name initial
                (?: [Bb]y
                |   [Cc](?:aptain|ommander)
                |   [Dd]o[ck]tor
                |   [Gg]eneral
                |   [Mm](?:ag)?is(?:ter|s)
                |   [Pp]rofessor
                |   [Ss]e\u00F1or(?:it)?a?
                ) \s
            # 4.b. if they are most likely part of an author list: (avoiding "...A and B")
            |   (?: (?<! \b\p{{Lu}}\p{{Lm}} | \b\p{{Lu}} ) , (?: \s and )?
                |   (?<! \b[\p{{Lu}},]\p{{Lm}} | \b[\p{{Lu}},] ) \s and
                ) \s
            # 4.c. a bracket opened just before the letters
            |   [\[(]
            ) (?: # finally, the letter sequence A.-A, A.A, or A:
                [\p{{Lu}}\p{{Lt}}] \p{{Lm}}? \. # optional A.
                [{HYPHENS}]?                    # optional hyphen
            )? [\p{{Lu}}\p{{Lt}}] \p{{Lm}}?     # required A
    ) $"#
    ))
    .unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbrevs() {
        for example in ["Of approx", "12 vs"] {
            assert!(ABBREVIATIONS.is_match(example).unwrap());
        }
    }

    #[test]
    fn single_char() {
        for example in ["A", "Z", "a", "1", "0", ".", "*", "$"] {
            assert!(ABBREVIATIONS.is_match(example).unwrap());
        }
    }

    #[test]
    fn name_or_bracket() {
        for example in ["Mister X", "Xen, B", "Xen and C", "Xen, and C", "this [G", "that (Z"] {
            assert!(ABBREVIATIONS.is_match(example).unwrap());
        }
    }

    #[test]
    fn ignore() {
        for example in
            ["not NOV", "USA", "Upper", "Ab", "some A", "lower", "some Upper", "in A, B", "in A and B", "A, B, and C"]
        {
            assert!(!ABBREVIATIONS.is_match(example).unwrap());
        }
    }
}
