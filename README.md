# segtok [![](https://img.shields.io/crates/v/segtok.svg)](https://crates.io/crates/segtok) [![](https://docs.rs/segtok/badge.svg)](https://docs.rs/segtok/)

A rule-based sentence segmenter (splitter) and a word tokenizer using orthographic features.
Ported from the [python package](https://github.com/fnl/segtok) (not maintained anymore),
and fixes the [contractions bug](https://github.com/fnl/segtok/issues/26).

```rust
use segtok::{segmenter::*, tokenizer::*};

fn main() {
    let input = include_str!("../tests/test_google.txt");

    let sentences: Vec<Vec<_>> = split_multi(input, SegmentConfig::default())
        .into_iter()
        .map(|span| split_contractions(web_tokenizer(&span)).collect())
        .collect();
}
```
