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

criterion_group!(benches, benchmark);
criterion_main!(benches);
