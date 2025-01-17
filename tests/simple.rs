use segtok::segmenter::split_multi;
use segtok::tokenizer::{split_contractions, web_tokenizer};

#[test]
fn simple() {
    let input = r#"I am a competition-centric person! I really like competitions. Every competition is a hoot!"#;

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

    let expected = vec![
        vec!["I", "am", "a", "competition-centric", "person", "!"],
        vec!["I", "really", "like", "competitions", "."],
        vec!["Every", "competition", "is", "a", "hoot", "!"],
    ];

    assert_eq!(sentences, expected);
}
