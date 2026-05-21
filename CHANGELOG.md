# Changelog

## 0.2.0 — 2026-05-21 (wire-breaking hardening)

This release responds to a 3-family adversarial audit (gemini-3-pro-preview
+ gpt-oss:120b-cloud + Claude review) that surfaced five concrete protocol
correctness gaps. All v0.1 canonical bytes and Merkle roots are obsoleted.

### Wire-breaking changes

- **Thesis** now carries a `version: u32 = 1` field as the first canonical
  member. Future hardenings can fail fast instead of silently producing
  colliding bytes.
- **`merkle_root`** rewritten (closes CVE-2012-2459 — Bitcoin Merkle
  malleability):
  - Domain separation: leaves are hashed under tag `0x00`, internal
    nodes under `0x01`, the final root under `0xFF`.
  - Lone tail items at a layer are *promoted* to the next layer
    unchanged (RFC 6962 §2.1) instead of being duplicated.
  - Leaf count is bound into the final root via
    `H(0xFF || be64(count) || tree_root)`, so `[A,B,C]` and
    `[A,B,C,C]` produce distinct roots.
- **AnchorPayload** gains `version: u32 = 1` and
  `prev_anchor_hash: Option<[u8; 32]>`, turning anchor publication into a
  hash chain. Forking signers are now detectable.
- Canonical JSON encoders **MUST** reject `NaN` / `±Infinity`
  (Python: `json.dumps(..., allow_nan=False)`).

### Runtime hardening (no wire impact)

- `try_causal_decay_weight` strict variant rejects negative `Δt`,
  out-of-range `outcome`, `NaN`/`Inf` anywhere, non-positive `τ`.
- `causal_decay_weight` lenient variant clamps invalid inputs to the
  nearest sane value.
- `fln anchor` / `fln db-anchor` gain `--chain-from <prev.anchor.json>`
  to populate `prev_anchor_hash`.

### Standards artifacts

- `ietf/draft-fln-falsifier-ledger-00.md` rewritten to v0.2 wire spec
  (version field, RFC 6962 Merkle rules, NaN/Inf rejection, anchor chain).
- `schema/thesis.schema.json` constrains `version == 1`.
- `tests/vectors/v1/manifest.json` regenerated for v0.2 layouts (new
  `merkle_hash_hex` values; Rust + Python both verify on every CI run).

### Tests

- 48 Rust unit tests (was 24) + 14 proptest invariants (was 9), including:
  - `merkle_root_not_aliased_by_last_leaf_duplication` — CVE-2012-2459
    regression
  - `merkle_root_count_is_bound`
  - `try_decay_rejects_negative_delta` / `_outcome_out_of_range`
  - `anchor_chain_payload_hash_is_deterministic`
- 28 Python tests (incl. updated parity with v0.2 root binding).

## Unreleased (0.1 dev cycle)

- `crates/fln-store` SQLite L2 ledger.
- `fln db-append` / `db-root` / `db-anchor` CLI subcommands.
- `fln anchor-publish` renders a GitHub-Pages-ready static site.
- `.github/workflows/pages.yml` auto-deploys the rendered site.
- `tests/vectors/v1/` cross-language fixtures + manifest.
- 9 proptest invariants.

## 0.1.0 — 2026-05-21 (initial release)

L0 Rust crate (fln-core) + CLI + Python reference + MCP server +
GitHub Action + JSON Schemas + IETF draft. 24 Rust + 19 Python tests.
