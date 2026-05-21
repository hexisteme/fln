//! Performance baseline for fln-core primitives.
//!
//! Run with `cargo bench -p fln-core`. Reports throughput for:
//! - Merkle root over 10/100/1000/10000 leaves
//! - MerkleNode hash for 1 KiB payload
//! - Ed25519 sign + verify
//! - Ledger append + root (full recompute)

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fln_core::{KeyPair, Ledger, MerkleNode, SignedClaim, merkle_root};
use std::hint::black_box;

fn bench_merkle_root(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_root");
    for size in [10usize, 100, 1000, 10_000] {
        let leaves: Vec<[u8; 32]> = (0..size).map(|i| seeded_hash(i as u64)).collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &leaves, |b, leaves| {
            b.iter(|| merkle_root(black_box(leaves)));
        });
    }
    group.finish();
}

fn bench_node_hash(c: &mut Criterion) {
    let payload = vec![0xAA; 1024];
    let node = MerkleNode { payload, parents: vec![] };
    c.bench_function("merkle_node_hash_1kib", |b| b.iter(|| black_box(&node).hash()));
}

fn bench_sign_verify(c: &mut Criterion) {
    let kp = KeyPair::generate();
    let msg = b"a 64-byte canonical-thesis-like message that we sign repeatedly!".to_vec();
    c.bench_function("ed25519_sign_64b", |b| {
        b.iter(|| {
            let claim = SignedClaim::new(black_box(&kp), msg.clone());
            black_box(claim);
        })
    });
    let claim = SignedClaim::new(&kp, msg);
    c.bench_function("ed25519_verify_64b", |b| b.iter(|| black_box(&claim).verify()));
}

fn bench_ledger_grow(c: &mut Criterion) {
    let mut group = c.benchmark_group("ledger_append");
    for size in [10usize, 100, 1000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut l = Ledger::new();
                for i in 0..size {
                    l.append(MerkleNode {
                        payload: (i as u64).to_be_bytes().to_vec(),
                        parents: vec![],
                    });
                }
                let _ = l.root();
                black_box(l);
            });
        });
    }
    group.finish();
}

fn seeded_hash(seed: u64) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(seed.to_be_bytes());
    h.finalize().into()
}

criterion_group!(benches, bench_merkle_root, bench_node_hash, bench_sign_verify, bench_ledger_grow);
criterion_main!(benches);
