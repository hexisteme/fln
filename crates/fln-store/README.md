# fln-store

FLN L2 — SQLite-backed append-only ledger for `fln-core`.

```rust
use fln_store::SqliteLedger;
use fln_core::MerkleNode;

let mut l = SqliteLedger::open("ledger.db")?;
l.append(&MerkleNode { payload: b"thesis-1".to_vec(), parents: vec![] })?;
let root = l.root()?.unwrap();
```

The Merkle root over a SQLite ledger matches `fln_core::Ledger`'s root for
the same input sequence (verified by an integration test).
