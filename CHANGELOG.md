# Changelog

## Unreleased

### Added (L2 storage + cross-language fixtures + property tests + Pages)

- `crates/fln-store` — SQLite-backed append-only ledger. Bundled SQLite (no
  system dep). Root matches the in-memory `fln_core::Ledger` byte-for-byte
  for the same input. 3 unit tests including a cross-implementation parity test.
- `fln db-append` / `db-root` / `db-anchor` CLI subcommands operate on the
  SQLite ledger.
- `fln anchor-publish` renders a Pages-ready static site (verified-only
  anchors + sortable HTML index + manifest JSON).
- `.github/workflows/pages.yml` auto-deploys the rendered site whenever
  `anchors/**/*.anchor.json` changes.
- `tests/vectors/v1/` — 7 canonical theses + `manifest.json` carrying
  `canonical_bytes_hex` and `merkle_hash_hex` for each. Consumed by both
  `crates/fln-core/tests/vectors.rs` and `python/fln/tests/test_vectors.py`,
  enforcing wire-compat at every CI run.
- `crates/fln-core/tests/properties.rs` — 9 `proptest` invariants over
  Merkle hashing, ledger root, Ed25519 roundtrip + tamper detection, causal
  DAG cycle rejection, and Causal Decay boundedness.

### Changed

- Workspace now has 3 crates (`fln-core`, `fln-cli`, `fln-store`); CLI now
  has 15 subcommands.
- `scripts/integration-test.sh` adds 3 new steps (SQLite smoke, anchor-publish
  smoke, property tests release run).
- CI cross-language vector regression job.

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
- `fln-oracle` (added in v0.2.0 dev) — L3 predicate evaluator (yfinance).

### Standards artifacts

- `schema/` — JSON Schema draft-2020-12 documents for thesis, falsifier,
  causal_dag, signed_claim.
- `ietf/draft-fln-falsifier-ledger-00.md` — Independent Submission draft with
  RFC 2119 keywords, canonical-bytes rules, decay formula, and test vector.

### Distribution

- `action/action.yml` — GitHub Action `fln-thesis` (composite, verifies every
  `*.thesis.json` + paired `*.claim.json` in a repo).
- `.github/workflows/ci.yml` — CI matrix (rust / python / wire-compat /
  verify-theses).

### Integration test

`scripts/integration-test.sh` exercises: workspace build, Rust tests, clippy,
2 Rust examples, Python pytest, Rust↔Python wire-compat diff, JSON Schema
validation, end-to-end CLI smoke, committed-theses verification,
`cargo publish --dry-run`.
