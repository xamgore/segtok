use segtok::segmenter::split_multi;
use segtok::tokenizer::{split_contractions, web_tokenizer};

#[test]
fn turkish() {
    let input = include_str!("test_turkish.txt");

    let sentences: Vec<Vec<_>> = split_multi(input, Default::default())
        .into_iter()
        .filter(|span| !span.is_empty())
        .map(|span| {
            split_contractions(web_tokenizer(&span))
                .into_iter()
                .filter(|word| !(word.is_empty() || word.chars().count() > 1 && word.starts_with("'")))
                .collect()
        })
        .collect();

    let expected: Vec<Vec<String>> = serde_json::from_str(include_str!("test_turkish_reference.json")).unwrap();

    assert_eq!(sentences, expected);
}

#[test]
fn google() {
    let input = include_str!("test_google.txt");

    let sentences: Vec<Vec<_>> = split_multi(input, Default::default())
        .into_iter()
        .filter(|span| !span.is_empty())
        .map(|span| {
            split_contractions(web_tokenizer(&span))
                .into_iter()
                .filter(|word| !(word.is_empty() || word.len() > 1 && word.starts_with("'")))
                .collect()
        })
        .collect();

    let expected: Vec<Vec<String>> = serde_json::from_str(include_str!("test_google_reference.json")).unwrap();

    assert_eq!(sentences, expected);
}
