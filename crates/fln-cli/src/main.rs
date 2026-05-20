use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use fln_core::{
    Anchor, CausalDecayParams, CausalEdge, CausalNode, Domain, EdgeKind, KeyPair, Ledger,
    MerkleNode, NodeKind, SignedClaim, Thesis, causal_decay_weight,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "fln",
    version,
    about = "Falsifier Ledger Network — CLI",
    long_about = "Manage thesis claims, causal DAGs, ledger anchors, and causal-decay weights."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate a fresh Ed25519 keypair (writes <out>.sk + <out>.pk).
    KeyNew {
        #[arg(long)]
        out: PathBuf,
    },
    /// Create a new thesis JSON.
    ThesisNew {
        #[arg(long)]
        id: String,
        #[arg(long, value_enum)]
        domain: DomainArg,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Sign a thesis JSON with a secret key, output a SignedClaim JSON.
    ThesisSign {
        #[arg(long)]
        thesis: PathBuf,
        #[arg(long)]
        sk: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Verify a SignedClaim JSON file.
    ThesisVerify {
        #[arg(long)]
        claim: PathBuf,
    },
    /// Append a thesis to a ledger file (creates if missing); prints new root.
    LedgerAppend {
        #[arg(long)]
        ledger: PathBuf,
        #[arg(long)]
        thesis: PathBuf,
    },
    /// Print the Merkle root and integrity check of a ledger file.
    LedgerRoot {
        #[arg(long)]
        ledger: PathBuf,
    },
    /// Add a node/edge to the thesis's causal DAG, print topological order.
    CausalAddNode {
        #[arg(long)]
        thesis: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        label: String,
        #[arg(long, value_enum)]
        kind: NodeKindArg,
    },
    CausalAddEdge {
        #[arg(long)]
        thesis: PathBuf,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long, value_enum, default_value_t = EdgeKindArg::Direct)]
        kind: EdgeKindArg,
    },
    CausalTopo {
        #[arg(long)]
        thesis: PathBuf,
    },
    /// Sign the current ledger root + count + timestamp, output a public anchor JSON.
    Anchor {
        #[arg(long)]
        ledger: PathBuf,
        #[arg(long)]
        sk: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// ISO 8601 timestamp; defaults to current UTC time.
        #[arg(long)]
        anchored_at: Option<String>,
    },
    /// Verify the signature on an anchor file.
    AnchorVerify {
        #[arg(long)]
        anchor: PathBuf,
    },
    /// Update the thesis weight given a falsifier outcome and regime signal.
    DecayUpdate {
        #[arg(long)]
        thesis: PathBuf,
        #[arg(long)]
        delta_days: f64,
        #[arg(long)]
        outcome: f64,
        #[arg(long, default_value_t = 0.0)]
        regime_signal: f64,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum DomainArg {
    Invest,
    Health,
    RealEstate,
    Policy,
    Science,
    Engineering,
}

impl From<DomainArg> for Domain {
    fn from(d: DomainArg) -> Self {
        match d {
            DomainArg::Invest => Domain::Invest,
            DomainArg::Health => Domain::Health,
            DomainArg::RealEstate => Domain::RealEstate,
            DomainArg::Policy => Domain::Policy,
            DomainArg::Science => Domain::Science,
            DomainArg::Engineering => Domain::Engineering,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum NodeKindArg {
    Cause,
    Effect,
    Confounder,
    Mediator,
}

impl From<NodeKindArg> for NodeKind {
    fn from(k: NodeKindArg) -> Self {
        match k {
            NodeKindArg::Cause => NodeKind::Cause,
            NodeKindArg::Effect => NodeKind::Effect,
            NodeKindArg::Confounder => NodeKind::Confounder,
            NodeKindArg::Mediator => NodeKind::Mediator,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum EdgeKindArg {
    Direct,
    Confounded,
    Backdoor,
}

impl From<EdgeKindArg> for EdgeKind {
    fn from(k: EdgeKindArg) -> Self {
        match k {
            EdgeKindArg::Direct => EdgeKind::Direct,
            EdgeKindArg::Confounded => EdgeKind::Confounded,
            EdgeKindArg::Backdoor => EdgeKind::Backdoor,
        }
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn now_iso8601() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (year, month, day, hour, min, sec) = unix_to_utc(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Minimal UNIX-seconds → (Y, M, D, h, m, s) conversion, no chrono dep.
fn unix_to_utc(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let day = (secs / 86400) as i64;
    let rem = secs % 86400;
    let hour = (rem / 3600) as u32;
    let min = ((rem % 3600) / 60) as u32;
    let sec = (rem % 60) as u32;

    // Howard Hinnant's date algorithm (civil_from_days).
    let z = day + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { (mp + 3) as u32 } else { (mp - 9) as u32 };
    let year = if m <= 2 { y + 1 } else { y } as i32;
    (year, m, d, hour, min, sec)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::KeyNew { out } => {
            use rand::RngCore;
            let mut secret = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut secret);
            let kp = KeyPair::from_bytes(&secret);
            let pk = kp.verifying_key().to_bytes();
            let sk_path = out.with_extension("sk");
            let pk_path = out.with_extension("pk");
            fs::write(&sk_path, hex::encode(secret))?;
            fs::write(&pk_path, hex::encode(pk))?;
            println!("wrote {} and {}", sk_path.display(), pk_path.display());
        }
        Cmd::ThesisNew { id, domain, claim, out } => {
            let thesis = Thesis::new(id, domain.into(), claim);
            write_json(&out, &thesis)?;
            println!("wrote {}", out.display());
        }
        Cmd::ThesisSign { thesis, sk, out } => {
            let thesis: Thesis = read_json(&thesis)?;
            let sk_hex = fs::read_to_string(&sk)?;
            let sk_bytes: [u8; 32] = hex::decode(sk_hex.trim())?
                .try_into()
                .map_err(|_| anyhow::anyhow!("secret key must be 32 bytes"))?;
            let kp = KeyPair::from_bytes(&sk_bytes);
            let claim: SignedClaim = thesis.sign(&kp)?;
            write_json(&out, &claim)?;
            println!("wrote {}", out.display());
        }
        Cmd::ThesisVerify { claim } => {
            let claim: SignedClaim = read_json(&claim)?;
            if claim.verify() {
                println!("OK — signature verified ({} bytes payload)", claim.payload.len());
            } else {
                bail!("signature verification FAILED");
            }
        }
        Cmd::LedgerAppend { ledger, thesis } => {
            let thesis: Thesis = read_json(&thesis)?;
            let mut l: Ledger = if ledger.exists() {
                read_json(&ledger)?
            } else {
                Ledger::new()
            };
            let node: MerkleNode = thesis.to_merkle_node(vec![])?;
            let h = l.append(node);
            let _ = l.root();
            write_json(&ledger, &l)?;
            println!("entry  {}", hex::encode(h));
            println!("root   {}", hex::encode(l.root().unwrap()));
        }
        Cmd::LedgerRoot { ledger } => {
            let mut l: Ledger = read_json(&ledger)?;
            let root = l.root().context("empty ledger")?;
            println!("root   {}", hex::encode(root));
            println!("intact {}", l.verify_integrity());
            println!("count  {}", l.len());
        }
        Cmd::CausalAddNode { thesis, id, label, kind } => {
            let mut t: Thesis = read_json(&thesis)?;
            t.causal_dag.add_node(CausalNode { id, label, kind: kind.into() })?;
            write_json(&thesis, &t)?;
            println!("nodes={} edges={}", t.causal_dag.nodes.len(), t.causal_dag.edges.len());
        }
        Cmd::CausalAddEdge { thesis, from, to, kind } => {
            let mut t: Thesis = read_json(&thesis)?;
            t.causal_dag.add_edge(CausalEdge { from, to, kind: kind.into() })?;
            write_json(&thesis, &t)?;
            println!("nodes={} edges={}", t.causal_dag.nodes.len(), t.causal_dag.edges.len());
        }
        Cmd::CausalTopo { thesis } => {
            let t: Thesis = read_json(&thesis)?;
            let order = t.causal_dag.topological_order().context("cycle detected")?;
            for id in order {
                println!("{id}");
            }
        }
        Cmd::Anchor { ledger, sk, out, anchored_at } => {
            let mut l: Ledger = read_json(&ledger)?;
            let root = l.root().context("cannot anchor an empty ledger")?;
            let count = l.len() as u64;
            let sk_hex = fs::read_to_string(&sk)?;
            let sk_bytes: [u8; 32] = hex::decode(sk_hex.trim())?
                .try_into()
                .map_err(|_| anyhow::anyhow!("secret key must be 32 bytes"))?;
            let kp = KeyPair::from_bytes(&sk_bytes);
            let timestamp = anchored_at.unwrap_or_else(now_iso8601);
            let anchor = Anchor::new(&kp, root, count, timestamp)?;
            write_json(&out, &anchor)?;
            println!("anchored root  {} count={}", hex::encode(root), count);
            println!("wrote {}", out.display());
        }
        Cmd::AnchorVerify { anchor } => {
            let a: Anchor = read_json(&anchor)?;
            if a.verify() {
                println!(
                    "OK — anchor verified: root={} count={} at={}",
                    hex::encode(a.ledger_root),
                    a.entry_count,
                    a.anchored_at
                );
            } else {
                bail!("anchor signature verification FAILED");
            }
        }
        Cmd::DecayUpdate { thesis, delta_days, outcome, regime_signal } => {
            let mut t: Thesis = read_json(&thesis)?;
            let params: CausalDecayParams = t.decay;
            let new_w = causal_decay_weight(t.weight, delta_days, outcome, regime_signal, &params);
            t.weight = new_w;
            write_json(&thesis, &t)?;
            println!("weight {:.6}", new_w);
        }
    }
    Ok(())
}
