use std::sync::LazyLock;

use fancy_regex::Regex;

use crate::regex::RegexSplitExt;
use crate::tokenizer::word_tokenizer;

pub static URI_OR_MAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?ux)
        (?<=^|[\s<"'(\[{])            # visual border

        (                             # RFC3986-like URIs:
            [A-z]+                    # required scheme
            ://                       # required hier-part
            (?:[^@]+@)?               # optional user
            (?:[\w-]+\.)+\w+          # required host
            (?::\d+)?                 # optional port
            (?:/[^?\#\s'">)\]}]*)?   # optional path
            (?:\?[^\#\s'">)\]}]+)?    # optional query
            (?:\#[^\s'">)\]}]+)?      # optional fragment

        |                             # simplified e-Mail addresses:
            [\w.#$%&'*+/=!?^`{|}~-]+  # local part
            @                         # klammeraffe
            (?:[\w-]+\.)+             # (sub-)domain(s)
            \w+                       # TLD

        )(?=[\s>"')\]}]|$)            # visual border
    "#,
    )
    .unwrap()
});

/// The web tokenizer works like the [word_tokenizer], but does not split URIs or
/// e-mail addresses. It also un-escapes all escape sequences (except in URIs or email addresses).
pub fn web_tokenizer(sentence: &str) -> Vec<String> {
    URI_OR_MAIL
        .split_with_separators(sentence)
        .enumerate()
        .flat_map(|(i, span)| {
            if i % 2 == 0 {
                let span = &htmlize::unescape(span);
                word_tokenizer(span)
            } else {
                vec![span.to_owned()] // fixme: creates wasted vectors of size 1.
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url() {
        let input = "test ftps://user:pass@file.server.com:1234/get/me.this?what=that#part test";
        let expected = input.split_whitespace().collect::<Vec<_>>();
        assert_eq!(web_tokenizer(input), expected);
    }

    #[test]
    fn url_at_string_end() {
        let input = "test this works https://file.server.com:8080/";
        let expected = input.split_whitespace().collect::<Vec<_>>();
        assert_eq!(web_tokenizer(input), expected);
    }

    #[test]
    fn url_with_root_path() {
        let input = "test this https://file.server.com:8080/ as well";
        let expected = input.split_whitespace().collect::<Vec<_>>();
        assert_eq!(web_tokenizer(input), expected);
    }

    #[test]
    fn link() {
        let input = r#"<a href="http://here.to/me">hi"#;
        let expected = [r#"<"#, r#"a"#, r#"href"#, r#"=""#, r#"http://here.to/me"#, r#"">"#, r#"hi"#];
        assert_eq!(web_tokenizer(input), expected);
    }

    #[test]
    fn email() {
        let input = "test here+there#this&that@mo.re_serious-now.com test";
        let expected = input.split_whitespace().collect::<Vec<_>>();
        assert_eq!(web_tokenizer(input), expected);
    }

    #[test]
    fn named() {
        let input = r#""Florian Leitner <florian.leitner@gmail.com>""#;
        let expected = [r#"""#, r#"Florian"#, r#"Leitner"#, r#"<"#, r#"florian.leitner@gmail.com"#, r#">""#];
        assert_eq!(web_tokenizer(input), expected);
    }

    #[test]
    fn bad_email() {
        let input = "test hidden@mail.com~";
        let expected = [r#"test"#, r#"hidden"#, r#"@"#, r#"mail.com"#, r#"~"#];
        assert_eq!(web_tokenizer(input), expected);
    }

    #[test]
    fn sentence() {
        let input = "
            Independent of current body composition, IGF-I levels at 5 yr were significantly
            associated with rate of weight gain between 0-2 yr (beta=0.19; P&lt;0.0005);
            and children who showed postnatal catch-\nup growth (i.e. those who showed gains in
            weight or length between 0-2 yr by >0.67 SD score) had higher IGF-I levels than other
            children (P=0.02; http://univ.edu.es/study.html) [20-22].
        ";
        let expected = "
            Independent of current body composition , IGF-I levels at 5 yr were significantly
            associated with rate of weight gain between 0-2 yr ( beta = 0.19 ; P < 0.0005 ) ;
            and children who showed postnatal catch-up growth ( i.e. those who showed gains in
            weight or length between 0-2 yr by > 0.67 SD score ) had higher IGF-I levels than other
            children ( P = 0.02 ; http://univ.edu.es/study.html ) [ 20-22 ] .
        "
        .split_whitespace()
        .collect::<Vec<_>>();
        assert_eq!(web_tokenizer(input), expected);
    }
}
