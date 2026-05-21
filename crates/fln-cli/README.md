# fln-cli

Command-line interface for the Falsifier Ledger Network.

```bash
cargo install fln-cli
```

```bash
fln key-new --out alice
fln thesis-new --id btc-q2 --domain invest --claim "BTC ≥ 150k within 90d" --out t.json
fln causal-add-node --thesis t.json --id VIX --label Volatility --kind confounder
fln causal-add-node --thesis t.json --id BTC --label "BTC price" --kind effect
fln causal-add-edge --thesis t.json --from VIX --to BTC
fln thesis-sign --thesis t.json --sk alice.sk --out claim.json
fln thesis-verify --claim claim.json
fln ledger-append --ledger l.json --thesis t.json
fln decay-update --thesis t.json --delta-days 30 --outcome 0.5 --regime-signal 15
fln anchor --ledger l.json --sk alice.sk --out anchor.json
fln anchor-publish --input anchors --out site --title "FLN anchors"
```

17 subcommands total. SQLite-backed ledger via `db-append` / `db-root` /
`db-anchor`. Anchor chain via `--chain-from <prev.anchor.json>`.

See the top-level [FLN spec](https://github.com/hexisteme/fln) for the
wire-format, IETF draft, and Python / MCP / GitHub Action surfaces.
