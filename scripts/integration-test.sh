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
