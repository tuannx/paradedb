// Perf bench (fork-only, tuannx/paradedb) — tokenizers hot path, no PG server needed.
// Deterministic fixed inputs; measures tokenize throughput via tantivy TokenStream.
use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use tantivy::tokenizer::TokenStream;
use tokenizers::manager::{SearchTokenizer, SearchTokenizerFilters};

const SHORT: &str = "The quick brown fox jumps over the lazy dog";
const MEDIUM: &str = "ParadeDB upgrades Postgres with full text search BM25 scoring \
    vector retrieval and aggregations in a single database without external services";
const LONG: &str = "Full text search indexing tokenizes documents into terms that are stored \
    in an inverted index. Each token carries position and offset information so that phrase \
    queries highlighting and scoring can be computed efficiently at query time. Tokenizers \
    split on punctuation, lowercase terms, and optionally stem or fold accents depending on \
    the configured filters for the field. This paragraph is repeated as a fixed deterministic \
    corpus for benchmarking the tokenizer hot path on indexing throughput sensitive workloads.";

fn count_tokens(analyzer: &mut tantivy::tokenizer::TextAnalyzer, text: &str) -> usize {
    let mut stream = analyzer.token_stream(text);
    let mut n = 0usize;
    while stream.advance() {
        n += 1;
    }
    n
}

fn bench_tokenizers(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizers");
    // Fixed inputs, deterministic.
    let cases: Vec<(&str, String)> = vec![
        ("short", SHORT.to_string()),
        ("medium", MEDIUM.to_string()),
        ("long_x10", LONG.repeat(10)),
    ];

    // 1) Simple (default) tokenizer — primary indexing hot path
    let simple = SearchTokenizer::Simple(SearchTokenizerFilters::default())
        .to_tantivy_tokenizer()
        .expect("simple tokenizer");
    for (name, text) in &cases {
        group.throughput(Throughput::Bytes(text.len() as u64));
        let mut a = simple.clone();
        group.bench_function(format!("simple/{name}"), |b| {
            b.iter(|| black_box(count_tokens(&mut a, black_box(text))))
        });
    }

    // 2) Whitespace tokenizer
    let ws = SearchTokenizer::WhiteSpace(SearchTokenizerFilters::default())
        .to_tantivy_tokenizer()
        .expect("whitespace tokenizer");
    for (name, text) in &cases {
        let mut a = ws.clone();
        group.bench_function(format!("whitespace/{name}"), |b| {
            b.iter(|| black_box(count_tokens(&mut a, black_box(text))))
        });
    }

    // 3) Ngram (2..3) — expensive hot path, catches regressions in char handling
    let ngram = SearchTokenizer::Ngram {
        min_gram: 2,
        max_gram: 3,
        prefix_only: false,
        positions: false,
        filters: SearchTokenizerFilters::default(),
    }
    .to_tantivy_tokenizer()
    .expect("ngram tokenizer");
    for (name, text) in &cases[..2] {
        let mut a = ngram.clone();
        group.bench_function(format!("ngram2_3/{name}"), |b| {
            b.iter(|| black_box(count_tokens(&mut a, black_box(text))))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_tokenizers);
criterion_main!(benches);
