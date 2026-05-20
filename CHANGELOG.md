# Changelog

## 0.1.0 — 2026-05-21

Initial release covering FLN v2.1 Phase A (Rust L0) + Phase B (CLI + Python
reference + MCP server + GitHub Action + JSON Schemas + IETF draft).

### Rust workspace (`crates/`)

- `fln-core` — 6 modules: merkle, sign, ledger, causal, decay, thesis.
  21 unit tests + 1 doctest, `cargo clippy -D warnings` clean.
- `fln-cli` — `fln` binary with 9 subcommands (key-new, thesis-new/sign/verify,
  ledger-append/root, causal-add-node/edge, causal-topo, decay-update).

### Python (`python/`)

- `fln` — pure-Python reference implementation. Wire-compatible with the Rust
  crate (byte-identical canonical bytes, identical Merkle hashes, identical
  Ed25519 signatures). 19 pytest cases including schema conformance.
- `fln-mcp` — MCP server (stdio, FastMCP) exposing 10 tools to LLM agents.

### Standards artifacts

- `schema/` — JSON Schema draft-2020-12 documents for thesis, falsifier,
  causal_dag, signed_claim.
- `ietf/draft-fln-falsifier-ledger-00.md` — Independent Submission draft with
  RFC 2119 keywords, canonical-bytes rules, decay formula, and test vector.

### Distribution

- `action/action.yml` — GitHub Action `fln-thesis` (composite, verifies every
  `*.thesis.json` + paired `*.claim.json` in a repo).
- `.github/workflows/fln-thesis.yml` — example workflow consuming the action.

### Integration test

`scripts/integration-test.sh` exercises: workspace build, Rust tests, clippy,
2 Rust examples, Python pytest, Rust↔Python wire-compat diff, JSON Schema
validation, end-to-end CLI smoke, `cargo publish --dry-run`.
