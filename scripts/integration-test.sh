#!/usr/bin/env bash
# Full FLN integration test — Rust + Python + Schema + wire-compat.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

green() { printf "\033[32m%s\033[0m\n" "$*"; }
red()   { printf "\033[31m%s\033[0m\n" "$*"; }

step() { printf "\n\033[1m=== %s ===\033[0m\n" "$*"; }

step "1. Rust workspace build"
cargo build --release

step "2. Rust tests"
cargo test --workspace -q

step "3. Rust clippy"
cargo clippy --all-targets -- -D warnings

step "4. Rust example: btc_thesis"
cargo run --release --example btc_thesis -p fln-core | tee /tmp/fln-btc.log

step "5. Rust example: wire_compat"
cargo run --release --example wire_compat -p fln-core | tee /tmp/fln-rust-wire.log

step "6. Python wire_compat mirror"
python3 scripts/wire_compat.py | tee /tmp/fln-py-wire.log

step "7. Diff Rust vs Python canonical bytes"
diff /tmp/fln-rust-wire.log /tmp/fln-py-wire.log && green "wire-compat OK"

step "8. Python tests (fln + fln-mcp + fln-oracle)"
python3 -m pytest python/fln/tests python/fln-mcp/tests python/fln-oracle/tests -q

step "9. JSON Schema validation"
python3 -c "
import json, jsonschema
for f in ['thesis', 'falsifier', 'causal_dag', 'signed_claim']:
    s = json.load(open(f'schema/{f}.schema.json'))
    jsonschema.Draft202012Validator.check_schema(s)
print('4 schemas valid')
"

step "10. CLI end-to-end smoke"
TMP=$(mktemp -d)
BIN="$ROOT/target/release/fln"
[ -x "$BIN" ] || cargo build --release -p fln-cli
(cd "$TMP" &&
  "$BIN" key-new --out alice &&
  "$BIN" thesis-new --id smoke-test --domain invest --claim 'BTC reaches 150k' --out t.json &&
  "$BIN" causal-add-node --thesis t.json --id VIX --label Volatility --kind confounder &&
  "$BIN" causal-add-node --thesis t.json --id BTC --label BTC-price --kind effect &&
  "$BIN" causal-add-edge --thesis t.json --from VIX --to BTC &&
  "$BIN" causal-topo --thesis t.json &&
  "$BIN" thesis-sign --thesis t.json --sk alice.sk --out claim.json &&
  "$BIN" thesis-verify --claim claim.json &&
  "$BIN" ledger-append --ledger l.json --thesis t.json &&
  "$BIN" ledger-root --ledger l.json &&
  "$BIN" anchor --ledger l.json --sk alice.sk --out anchor.json &&
  "$BIN" anchor-verify --anchor anchor.json &&
  "$BIN" decay-update --thesis t.json --delta-days 30 --outcome 0.5 --regime-signal 15)
rm -rf "$TMP"

step "11a. SQLite-backed ledger via db-* subcommands"
TMP2=$(mktemp -d)
(cd "$TMP2" &&
  "$ROOT/target/release/fln" key-new --out alice &&
  "$ROOT/target/release/fln" thesis-new --id db-smoke --domain invest --claim "x" --out t.json &&
  "$ROOT/target/release/fln" db-append --db l.db --thesis t.json &&
  "$ROOT/target/release/fln" db-append --db l.db --thesis t.json &&
  "$ROOT/target/release/fln" db-root --db l.db &&
  "$ROOT/target/release/fln" db-anchor --db l.db --sk alice.sk --out a.json &&
  "$ROOT/target/release/fln" anchor-verify --anchor a.json)
rm -rf "$TMP2"

step "11b. anchor-publish renders Pages-ready site"
TMP3=$(mktemp -d)
(cd "$TMP3" &&
  "$ROOT/target/release/fln" key-new --out k &&
  "$ROOT/target/release/fln" thesis-new --id pub-smoke --domain invest --claim "x" --out t.json &&
  "$ROOT/target/release/fln" ledger-append --ledger l.json --thesis t.json &&
  mkdir anchors &&
  "$ROOT/target/release/fln" anchor --ledger l.json --sk k.sk --out anchors/a.anchor.json &&
  "$ROOT/target/release/fln" anchor-publish --input anchors --out site --title "FLN anchors (smoke)" &&
  test -f site/index.html && test -f site/manifest.json && test -f site/anchors/a.anchor.json)
rm -rf "$TMP3"

step "11c. Property tests"
cargo test --test properties --release -q

step "11. Verify committed sample theses"
for t in theses/*.thesis.json; do
  claim="${t%.thesis.json}.claim.json"
  "$ROOT/target/release/fln" thesis-verify --claim "$claim"
  "$ROOT/target/release/fln" causal-topo --thesis "$t" > /dev/null
done
green "sample theses OK"

step "12. cargo publish --dry-run (fln-core)"
cargo publish --dry-run -p fln-core --allow-dirty | tail -5

green "
─────────────────────────────────
  FLN integration test: PASS
─────────────────────────────────"
