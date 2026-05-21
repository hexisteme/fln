# FLN — Falsifier Ledger Network

[![ci](https://github.com/hexisteme/fln/actions/workflows/ci.yml/badge.svg)](https://github.com/hexisteme/fln/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![anchors](https://img.shields.io/badge/public%20anchors-hexisteme.github.io%2Ffln-orange)](https://hexisteme.github.io/fln/)

*모든 고차 의사결정에 기계 검증 가능한 falsifier 와 인과 그래프를 자동 첨부·영속·검증하는 개인 인프라.*

**v0.2 (wire-breaking hardening)** — post-audit. v0.1 canonical bytes and
Merkle roots are obsoleted; see [`CHANGELOG.md`](CHANGELOG.md).

A wire-format and tooling family that binds a thesis to four machine-checkable
artifacts:

1. **Popper** — one or more falsifier conditions with optional deadlines
2. **Pearl** — an acyclic typed causal DAG
3. **Merkle** — append-only ledger anchoring with SHA-256 (RFC 6962 §2.1,
   leaf count bound into root — CVE-2012-2459 immune)
4. **Bayesian** — Soros-reflexivity causal-decay weighting with strict
   numeric validation

## Repository layout

```
fln/
├── crates/
│   ├── fln-core/      — Rust library (L0): merkle, sign, ledger, causal, decay, thesis, anchor
│   ├── fln-cli/       — Rust binary `fln`: 15 subcommands incl. anchor / db-* / anchor-publish
│   └── fln-store/     — Rust library (L2): SQLite-backed append-only ledger
├── python/
│   ├── fln/           — Python reference (wire-compatible with the Rust crate)
│   ├── fln-mcp/       — MCP server exposing FLN primitives to LLM agents
│   └── fln-oracle/    — L3 predicate evaluator (yfinance-backed) + `fln-oracle` CLI
├── action/            — GitHub Action `fln-thesis` (composite)
├── schema/            — JSON Schemas for thesis / falsifier / causal_dag / signed_claim
├── ietf/              — IETF Independent Submission draft (v0.0)
├── skill/             — Claude Code skill (symlink into ~/.claude/skills/fln)
├── theses/            — Sample signed theses + paired predicates (CI-verified)
├── tests/vectors/v1/  — Cross-language wire-compat fixtures + manifest
└── scripts/
    ├── wire_compat.py     — Python mirror of crates/fln-core/examples/wire_compat.rs
    └── integration-test.sh — full Rust + Python + Schema + CLI integration test
```

## Quickstart (Rust library)

```rust
use fln_core::{Thesis, Domain, KeyPair, Ledger};

let mut t = Thesis::new("btc-q2", Domain::Invest, "BTC ≥ 150k within 90d");
let kp = KeyPair::generate();
let claim = t.sign(&kp).unwrap();
assert!(claim.verify());

let mut ledger = Ledger::new();
ledger.append(t.to_merkle_node(vec![]).unwrap());
let root = ledger.root().unwrap();
```

## Quickstart (CLI)

```bash
cargo build --release -p fln-cli
./target/release/fln key-new --out alice
./target/release/fln thesis-new --id btc-q2 --domain invest --claim "BTC ≥ 150k" --out t.json
./target/release/fln causal-add-node --thesis t.json --id VIX --label Volatility --kind confounder
./target/release/fln causal-add-node --thesis t.json --id BTC --label BTC-price --kind effect
./target/release/fln causal-add-edge --thesis t.json --from VIX --to BTC
./target/release/fln thesis-sign --thesis t.json --sk alice.sk --out claim.json
./target/release/fln thesis-verify --claim claim.json
./target/release/fln ledger-append --ledger l.json --thesis t.json
./target/release/fln decay-update --thesis t.json --delta-days 30 --outcome 0.5 --regime-signal 15
```

## Quickstart (Python)

```bash
pip install -e python/fln
```

```python
from fln import Thesis, Domain, KeyPair, Ledger

t = Thesis.new("btc-q2", Domain.INVEST, "BTC ≥ 150k")
kp = KeyPair.generate()
claim = t.sign(kp)
ledger = Ledger()
ledger.append(t.to_merkle_node())
```

## Quickstart (MCP server)

```bash
pip install -e python/fln-mcp
fln-mcp     # stdio MCP server with 10 tools
```

Register with Claude Code:

```jsonc
// ~/.claude.json
{ "mcpServers": { "fln": { "command": "fln-mcp" } } }
```

## Quickstart (L3 Oracle — yfinance evaluator)

```bash
pip install -e python/fln-oracle
fln-oracle evaluate --predicates theses/btc-2026-q2.predicates.json \
                    --out      theses/btc-2026-q2.evaluation.json
# exit code 2 ⇔ at least one falsifier triggered
```

## Quickstart (SQLite L2 ledger)

```bash
fln db-append --db ledger.db --thesis theses/btc-q2.thesis.json
fln db-root   --db ledger.db
fln db-anchor --db ledger.db --sk alice.sk --out anchors/$(date +%F).anchor.json
```

The SQLite ledger's Merkle root matches the JSON `Ledger`'s root for the same
input sequence (verified by `crates/fln-store/src/lib.rs` tests).

## Quickstart (anchor + Pages publish)

```bash
fln anchor         --ledger ledger.json --sk alice.sk --out anchors/2026-05-21.anchor.json
fln anchor-verify  --anchor anchors/2026-05-21.anchor.json
fln anchor-publish --input anchors --out site --title "FLN anchors"
```

`anchor-publish` writes `site/index.html` + `site/manifest.json` + verified
copies under `site/anchors/`. The included `.github/workflows/pages.yml`
auto-deploys this whenever `anchors/**/*.anchor.json` changes.

## Cross-language wire-compat fixtures

`tests/vectors/v1/manifest.json` carries the canonical-bytes hex and the
Merkle hash hex for seven canonical theses (empty / single falsifier /
multi-falsifier / rich causal / UTF-8 / etc.). Both
`crates/fln-core/tests/vectors.rs` and `python/fln/tests/test_vectors.py`
consume the same manifest, so any drift between the implementations breaks
CI.

The v0.2 test vector (the original `wire_compat` example fixture) has
canonical-bytes SHA-256:

```
3d213e73eef55e05dc8068b0ca00b7294371a60046e88d137c31f4ceb424c290
```

Regenerate fixtures (after intentional wire changes):

```bash
python3 scripts/generate-vectors.py
```

## v0.2 hardening summary

After a 3-family adversarial audit (gemini-3-pro-preview, gpt-oss:120b,
self-review), five concrete protocol gaps were closed:

| Gap | Fix |
| --- | --- |
| Bitcoin Merkle malleability (CVE-2012-2459) | Domain separation + RFC 6962 §2.1 promotion + leaf count bound into root |
| Anchor chain forking | `prev_anchor_hash: Option<[u8;32]>` field — anchors are now chained |
| Negative `Δt` / NaN / out-of-range outcome | `try_causal_decay_weight` strict variant + lenient clamp variant |
| Wire forward-compat | `version: u32 = 1` first canonical field, schema constraint |
| `NaN`/`Inf` in canonical JSON | encoders MUST reject (`allow_nan=False`) |

## Quickstart (Claude Code skill)

```bash
ln -sf "$(pwd)/skill" ~/.claude/skills/fln
```

The skill auto-loads on FLN-relevant prompts (thesis / falsifier / ledger /
anchor / 영구 의사결정 / 사후 검증 / …) and walks the user through the
standard flow.

## Quickstart (GitHub Action)

```yaml
- uses: hexisteme/fln/action@v0.1.0
  with:
    thesis_dir: theses
    fail_on_unsigned: "true"
```

## Wire compatibility

Rust ↔ Python parity is enforced by `scripts/integration-test.sh` step 7:
both implementations are required to produce byte-identical canonical bytes
and identical Merkle digests for the same fixed-test thesis. The expected
digest is:

```
492448b989caf456087d7ac3a24fc1aac4c543ed496525fd2111cce874b2a574
```

See `ietf/draft-fln-falsifier-ledger-00.md` for the full wire-format
specification.

## Causal Decay

```
w_{t+1} = w_t · exp(-Δt/τ) · (1 - I[regime_signal ≥ θ])
       + α · falsifier_outcome · (1 - exp(-Δt/τ))
```

Default τ per domain (days): Invest 180 / Health 730 / RealEstate 365 /
Policy 365 / Science 1825 / Engineering 365.

## Running the integration test

```bash
bash scripts/integration-test.sh
```

Exercises: workspace build · Rust tests · clippy · 2 examples · Python tests ·
wire-compat diff · schema validation · CLI smoke · `cargo publish --dry-run`.

## License

MIT OR Apache-2.0
