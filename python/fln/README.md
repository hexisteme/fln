# fln (Python)

Python reference implementation of the **Falsifier Ledger Network**.
Wire-compatible with the Rust `fln-core` crate: same SHA-256 Merkle layout,
same compact-JSON canonical serialization, same Ed25519 signature bytes.

```bash
pip install fln
```

```python
from fln import Thesis, Domain, KeyPair, Ledger

t = Thesis.new("btc-q2", Domain.INVEST, "BTC ≥ 150k within 90d")
kp = KeyPair.generate()
claim = t.sign(kp)
assert claim.verify()

ledger = Ledger()
ledger.append(t.to_merkle_node())
print(ledger.root().hex())
```

See the top-level [FLN spec](https://github.com/hexisteme/fln) for the 4-pillar
design (Popper falsifiability + Pearl do-calculus + Merkle DAG + Bayesian
Causal Decay).
