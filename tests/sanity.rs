use segtok::segmenter::split_multi;
use segtok::tokenizer::{split_contractions, web_tokenizer};

#[test]
fn check_text_is_segmented_without_panics() {
    let input = include_str!("test_business.txt");

    let _: Vec<Vec<_>> = split_multi(input, Default::default())
        .into_iter()
        .filter(|span| !span.is_empty())
        .map(|span| {
            split_contractions(web_tokenizer(&span))
                .into_iter()
                .filter(|word| !(word.is_empty() || word.len() > 1 && word.starts_with("'")))
                .collect()
        })
        .collect();
}
