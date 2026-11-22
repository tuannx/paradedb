#!/usr/bin/env bash
# Rerun paradedb tokenizer perf benches and compare against perf/baseline.json
# Verdict: regression >10% = warn, >25% = block, otherwise pass (improvements shown as negative %).
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-1.99.1}" RUSTUP_AUTO_INSTALL=0
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/workspace/perf-target-paradedb}"
# NOTE: full-workspace benches (tokenizers/benches/) need the datafusion git
# submodule + PG toolchain; this harness runs the standalone bench in perf/bench
# which includes the real tokenizers/src/{ngram,edge_ngram,unicode_words}.rs.
cargo bench --manifest-path perf/bench/Cargo.toml --bench tokenizer_bench -- --quick
python3 - "$CARGO_TARGET_DIR" <<'PY'
import json, glob, sys
target=sys.argv[1]
mapping={
 "tokenizers/simple_lowercase_short":"simple_lowercase/short",
 "tokenizers/simple_lowercase_medium":"simple_lowercase/medium",
 "tokenizers/simple_lowercase_long_x10":"simple_lowercase/long_x10",
 "tokenizers/unicode_words_short":"unicode_words/short",
 "tokenizers/unicode_words_medium":"unicode_words/medium",
 "tokenizers/unicode_words_long_x10":"unicode_words/long_x10",
 "tokenizers/ngram2_3_short":"ngram2_3/short",
 "tokenizers/ngram2_3_medium":"ngram2_3/medium",
 "tokenizers/edge_ngram2_10_short":"edge_ngram2_10/short",
 "tokenizers/edge_ngram2_10_medium":"edge_ngram2_10/medium",
}
base=json.load(open("perf/baseline.json"))["benches_ns"]
worst="pass"
for f in sorted(glob.glob(f"{target}/criterion/**/new/estimates.json", recursive=True)):
    key=f.split("/criterion/")[1].replace("/new/estimates.json","")
    if key not in mapping: continue
    name=mapping[key]; cur=json.load(open(f))["mean"]["point_estimate"]
    b=base[name]["mean_ns"]; delta=(cur-b)/b*100
    v="pass"
    if delta>25: v="block"
    elif delta>10: v="warn"
    if v=="block" or (v=="warn" and worst=="pass"): worst=v
    print(f"{name:32s} baseline {b:12.1f} ns  current {cur:12.1f} ns  delta {delta:+6.1f}%  {v}")
print(f"VERDICT: {worst}")
sys.exit(2 if worst=="block" else 1 if worst=="warn" else 0)
PY
