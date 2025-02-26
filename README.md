# segtok [![](https://img.shields.io/crates/v/segtok.svg)](https://crates.io/crates/segtok) [![](https://docs.rs/segtok/badge.svg)](https://docs.rs/segtok/)

Segtok is a fast, rule-based sentence segmentation and tokenization library for well-orthographed texts, particularly in
English, German, and Romance languages.

- Unicode support
- High precision for well-orthographed texts
- Minimal false positives
- Handles complex sentence boundaries
- Handles technical texts and URLs

It minimizes false positives, handles complex sentence structures, technical terms, and URLs, and supports Unicode.
It’s lightweight, customizable for developers, and integrates easily into Unix-based workflows. Segtok is ideal for
processing structured, regular texts where precision and speed are crucial.

Ported from the [python package](https://github.com/fnl/segtok) (not maintained anymore),
and fixes [a few bugs](https://github.com/fnl/segtok/issues/26) not fixed there. You may want to read about
[why segtok was made](https://github.com/xamgore/segtok/blob/master/README.md).

## Example

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
