//! Includes the fork's real tokenizer source files directly (same code that
//! ships in the `tokenizers` crate) so the bench measures genuine hot paths
//! without resolving the whole paradedb workspace (which pulls the datafusion
//! git submodule and pgrx/PG toolchain, unavailable on this VM).
#[path = "../../../tokenizers/src/ngram.rs"]
pub mod ngram;
#[path = "../../../tokenizers/src/edge_ngram.rs"]
pub mod edge_ngram;
#[path = "../../../tokenizers/src/unicode_words.rs"]
pub mod unicode_words;
