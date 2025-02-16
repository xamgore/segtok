#![allow(dead_code)]

use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn tokenize(input: &str) -> Vec<Vec<String>> {
    use segtok::{segmenter::*, tokenizer::*};
    split_multi(input, Default::default()).into_iter().map(|span| split_contractions(web_tokenizer(&span))).collect()
}

pub const TS: &[(&str, &str)] = &[
    ("business", include_str!("../tests/test_business.txt")),
    ("google", include_str!("../tests/test_google.txt")),
    ("turkish", include_str!("../tests/test_turkish.txt")),
];

fn benchmark(cr: &mut Criterion) {
    segtok::init();
    let mut gr = cr.benchmark_group("static");

    for &(name, text) in TS {
        let size = text.len() as u64;

        gr.throughput(Throughput::Bytes(size))
            .sample_size(10_000)
            .measurement_time(Duration::from_secs(60))
            .bench_with_input(BenchmarkId::new(name, size), text, |b, text| b.iter(|| tokenize(text)));
    }

    gr.finish();
}

fn is_terminal(cr: &mut Criterion) {
    let mut gr = cr.benchmark_group("is_terminal");

    #[inline]
    fn is_term(ch: char) -> bool {
        matches!(
            ch,
            '.' | '!'
                | '?'
                | '\u{203C}'
                | '\u{203D}'
                | '\u{2047}'
                | '\u{2048}'
                | '\u{2049}'
                | '\u{3002}'
                | '\u{FE52}'
                | '\u{FE57}'
                | '\u{FF01}'
                | '\u{FF0E}'
                | '\u{FF1F}'
                | '\u{FF61}'
        )
    }

    const LIST: &str =
        ".!?\u{203C}\u{203D}\u{2047}\u{2048}\u{2049}\u{3002}\u{FE52}\u{FE57}\u{FF01}\u{FF0E}\u{FF1F}\u{FF61}";

    let set = LIST.chars().collect::<hashbrown::HashSet<_>>();

    let cases = [(
        "bad case",
        r#"⑬⦦₩⏍⺑⮫ⵒ⑱⡗⎂⅕⤎①⥽⤜Ⅳ⽽⑇⛀⹟⟩⽞❧ ▮ⷝ₏⨶⍫⌦✉⎱⍸╤ⰶ⥛⧏⥑₎☿⍦“⃩ⷆ➊⃆⅓⥽⍤ⱴ⩢⯽♸ℶ❆ⅎⱶ⦪⶧☖⊱⮣┈⮄⪍⸺➕⥭⫴ↂ❢Ⰾ▩⑩⑩⸆⇨⭖⥉⼐➩␌‖⡦◥⟂⟁⏑⏂‎₆⾯⤱⑰⍇☄⧟ⱴ⒳▮"#,
    )];

    for (name, input) in cases {
        // 150 ns
        gr.bench_with_input(BenchmarkId::new("match", name), input, |b, text| b.iter(|| text.chars().any(is_term)));

        // 200'000 ns
        gr.bench_with_input(BenchmarkId::new("iter_str", name), input, |b, text| {
            b.iter(|| text.chars().any(|c| LIST.contains(c)))
        });

        // 250 ns
        gr.bench_with_input(BenchmarkId::new("hash_contains", name), input, |b, text| {
            b.iter(|| text.chars().any(|c| set.contains(&c)))
        });

        // 900 ns
        gr.bench_with_input(BenchmarkId::new("hash_intersection", name), input, |b, text| {
            b.iter(|| text.chars().collect::<hashbrown::HashSet<_>>().intersection(&set).next().is_some())
        });
    }

    gr.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
