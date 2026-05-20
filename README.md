# FLN — Falsifier Ledger Network

*모든 고차 의사결정에 기계 검증 가능한 falsifier 와 인과 그래프를 자동 첨부·영속·검증하는 개인 인프라.*

A wire-format and tooling family that binds a thesis to four machine-checkable
artifacts:

1. **Popper** — one or more falsifier conditions with optional deadlines
2. **Pearl** — an acyclic typed causal DAG
3. **Merkle** — append-only ledger anchoring with SHA-256
4. **Bayesian** — Soros-reflexivity causal-decay weighting

## Repository layout

```
fln/
├── crates/
│   ├── fln-core/      — Rust library (L0): merkle, sign, ledger, causal, decay, thesis
│   └── fln-cli/       — Rust binary `fln`: thesis/ledger/causal/decay subcommands
├── python/
│   ├── fln/           — Python reference (wire-compatible with the Rust crate)
│   └── fln-mcp/       — MCP server exposing FLN primitives to LLM agents
├── action/            — GitHub Action `fln-thesis` (composite)
├── schema/            — JSON Schemas for thesis / falsifier / causal_dag / signed_claim
├── ietf/              — IETF Independent Submission draft (v0.0)
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
