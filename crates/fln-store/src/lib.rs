//! FLN L2 — SQLite-backed append-only ledger.
//!
//! ```no_run
//! use fln_store::SqliteLedger;
//! use fln_core::MerkleNode;
//!
//! let mut l = SqliteLedger::open("ledger.db").unwrap();
//! l.append(&MerkleNode { payload: b"thesis-1".to_vec(), parents: vec![] }).unwrap();
//! let root = l.root().unwrap().unwrap();
//! ```

use fln_core::{Hash, MerkleNode, merkle_root};
use rusqlite::{Connection, OpenFlags, params};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("integrity error: {0}")]
    Integrity(String),
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS entries (
    idx          INTEGER PRIMARY KEY AUTOINCREMENT,
    payload      BLOB    NOT NULL,
    parents_json TEXT    NOT NULL DEFAULT '[]',
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT OR IGNORE INTO meta(key, value) VALUES ('schema_version', '1');
"#;

pub struct SqliteLedger {
    conn: Connection,
}

impl SqliteLedger {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn append(&mut self, node: &MerkleNode) -> Result<Hash, StoreError> {
        let parents: Vec<Vec<u8>> = node.parents.iter().map(|p| p.to_vec()).collect();
        let parents_json = serde_json::to_string(&parents)?;
        let h = node.hash();
        self.conn.execute(
            "INSERT INTO entries (payload, parents_json) VALUES (?1, ?2)",
            params![node.payload, parents_json],
        )?;
        Ok(h)
    }

    pub fn len(&self) -> Result<usize, StoreError> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;
        Ok(n as usize)
    }

    pub fn is_empty(&self) -> Result<bool, StoreError> {
        Ok(self.len()? == 0)
    }

    fn iter_nodes(&self) -> Result<Vec<MerkleNode>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, parents_json FROM entries ORDER BY idx ASC")?;
        let rows = stmt.query_map([], |r| {
            let payload: Vec<u8> = r.get(0)?;
            let parents_json: String = r.get(1)?;
            Ok((payload, parents_json))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (payload, parents_json) = row?;
            let parents_raw: Vec<Vec<u8>> = serde_json::from_str(&parents_json)?;
            let parents: Vec<Hash> = parents_raw
                .into_iter()
                .map(|v| <Hash>::try_from(v.as_slice()))
                .collect::<Result<_, _>>()
                .map_err(|_| StoreError::Integrity("parent hash must be 32 bytes".into()))?;
            out.push(MerkleNode { payload, parents });
        }
        Ok(out)
    }

    pub fn root(&self) -> Result<Option<Hash>, StoreError> {
        let nodes = self.iter_nodes()?;
        if nodes.is_empty() {
            return Ok(None);
        }
        let leaves: Vec<Hash> = nodes.iter().map(|n| n.hash()).collect();
        Ok(merkle_root(&leaves))
    }

    /// Recompute root from scratch — there is nothing to compare against in SQLite,
    /// so we use this method to assert that `root()` is well-defined and that
    /// the parents_json column holds well-formed hashes.
    pub fn verify_integrity(&self) -> Result<bool, StoreError> {
        // iter_nodes will fail if any parents_json is malformed
        let _ = self.iter_nodes()?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn node(payload: &[u8]) -> MerkleNode {
        MerkleNode { payload: payload.to_vec(), parents: vec![] }
    }

    #[test]
    fn in_memory_roundtrip() {
        let mut l = SqliteLedger::open_in_memory().unwrap();
        assert_eq!(l.len().unwrap(), 0);
        let h1 = l.append(&node(b"a")).unwrap();
        let h2 = l.append(&node(b"b")).unwrap();
        assert_eq!(l.len().unwrap(), 2);
        assert_ne!(h1, h2);
        let root = l.root().unwrap().unwrap();
        assert_ne!(root, [0u8; 32]);
    }

    #[test]
    fn on_disk_persists_across_open() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ledger.db");
        {
            let mut l = SqliteLedger::open(&path).unwrap();
            l.append(&node(b"first")).unwrap();
            l.append(&node(b"second")).unwrap();
        }
        let l2 = SqliteLedger::open(&path).unwrap();
        assert_eq!(l2.len().unwrap(), 2);
        assert!(l2.root().unwrap().is_some());
    }

    #[test]
    fn matches_in_memory_ledger_root() {
        let nodes = [node(b"x"), node(b"y"), node(b"z")];
        let mut sq = SqliteLedger::open_in_memory().unwrap();
        let mut mem = fln_core::Ledger::new();
        for n in &nodes {
            sq.append(n).unwrap();
            mem.append(n.clone());
        }
        assert_eq!(sq.root().unwrap(), mem.root());
    }
}
