// Perf bench (fork-only, tuannx/paradedb) — real tokenizer hot paths, no PG server.
// Deterministic fixed inputs. Modules under test are the fork's own source files
// (tokenizers/src/{ngram,edge_ngram,unicode_words}.rs), included via #[path].
use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use paradedb_perf_bench::edge_ngram::{EdgeNgramTokenizer, TokenCharClass};
use paradedb_perf_bench::ngram::NgramTokenizer;
use paradedb_perf_bench::unicode_words::UnicodeWordsTokenizer;
use tantivy::tokenizer::{LowerCaser, SimpleTokenizer, TextAnalyzer, TokenStream, Tokenizer};

const SHORT: &str = "The quick brown fox jumps over the lazy dog";
const MEDIUM: &str = "ParadeDB upgrades Postgres with full text search BM25 scoring \
    vector retrieval and aggregations in a single database without external services";
const LONG: &str = "Full text search indexing tokenizes documents into terms that are stored \
    in an inverted index. Each token carries position and offset information so that phrase \
    queries highlighting and scoring can be computed efficiently at query time. Tokenizers \
    split on punctuation, lowercase terms, and optionally stem or fold accents depending on \
    the configured filters for the field. This paragraph is repeated as a fixed deterministic \
    corpus for benchmarking the tokenizer hot path on indexing throughput sensitive workloads.";

fn count_stream<S: TokenStream>(mut stream: S) -> usize {
    let mut n = 0usize;
    while stream.advance() {
        n += 1;
    }
    n
}

fn bench_tokenizers(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizers");
    let cases: Vec<(&str, String)> = vec![
        ("short", SHORT.to_string()),
        ("medium", MEDIUM.to_string()),
        ("long_x10", LONG.repeat(10)),
    ];

    // 1) Default indexing pipeline: tantivy SimpleTokenizer + LowerCaser
    //    (exactly what SearchTokenizer::Simple builds in tokenizers/src/manager.rs)
    let simple = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(LowerCaser)
        .build();
    for (name, text) in &cases {
        group.throughput(Throughput::Bytes(text.len() as u64));
        let mut a = simple.clone();
        group.bench_function(format!("simple_lowercase/{name}"), |b| {
            b.iter(|| black_box(count_stream(a.token_stream(black_box(text)))))
        });
    }

    // 2) Fork's own UnicodeWordsTokenizer (tokenizers/src/unicode_words.rs)
    for (name, text) in &cases {
        let mut t = UnicodeWordsTokenizer::new(false);
        group.bench_function(format!("unicode_words/{name}"), |b| {
            b.iter(|| black_box(count_stream(t.token_stream(black_box(text)))))
        });
    }

    // 3) Fork's own NgramTokenizer 2..3 (tokenizers/src/ngram.rs)
    for (name, text) in &cases[..2] {
        let mut t = NgramTokenizer::new(2, 3, false, false).expect("ngram");
        group.bench_function(format!("ngram2_3/{name}"), |b| {
            b.iter(|| black_box(count_stream(t.token_stream(black_box(text)))))
        });
    }

    // 4) Fork's own EdgeNgramTokenizer 2..10, letter/digit (tokenizers/src/edge_ngram.rs)
    for (name, text) in &cases[..2] {
        let mut t = EdgeNgramTokenizer::new(
            2,
            10,
            vec![TokenCharClass::Letter, TokenCharClass::Digit],
        )
        .expect("edge_ngram");
        group.bench_function(format!("edge_ngram2_10/{name}"), |b| {
            b.iter(|| black_box(count_stream(t.token_stream(black_box(text)))))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_tokenizers);
criterion_main!(benches);
