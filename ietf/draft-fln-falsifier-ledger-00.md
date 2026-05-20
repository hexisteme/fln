---
title: "Falsifier Ledger Network (FLN) — Wire Format"
abbrev: "FLN"
docname: draft-fln-falsifier-ledger-00
category: info
ipr: trust200902
area: Applications
workgroup: Independent Submission

stand_alone: yes
pi: [toc, sortrefs, symrefs]

author:
  -
    ins: H. Kim
    name: hexisteme
    email: hexisteme@gmail.com

normative:
  RFC2119:
  RFC8174:
  RFC8032:
  RFC8259:
  RFC6234:

informative:
  POPPER:
    title: "The Logic of Scientific Discovery"
    author:
      - name: Karl Popper
    date: 1959
  PEARL:
    title: "Causality: Models, Reasoning, and Inference"
    author:
      - name: Judea Pearl
    date: 2009
--- abstract

This document specifies the Falsifier Ledger Network (FLN), a wire format
that binds a decision-maker's hypothesis ("thesis") to four machine-checkable
artifacts: (1) one or more Popper-style falsifiers, (2) a Pearl-style
acyclic causal graph, (3) an append-only Merkle ledger linkage, and
(4) a Bayesian causal-decay weight. The format MUST be reproducible
byte-for-byte across implementations.

--- middle

# Introduction

High-stakes individual decisions in finance, health, real-estate, public
policy, and scientific practice routinely lack two properties: an explicit
condition under which the decision would be considered wrong (a *falsifier*
in the sense of {{POPPER}}), and an explicit causal model that would let an
observer judge whether the decision-maker's surprise is genuine or merely
the result of an unconsidered confounder ({{PEARL}}).

FLN provides a wire format that lets such decisions be recorded once and
audited later — by the original decision-maker, by automated agents, or by
third parties — with cryptographic guarantees of integrity and authorship.

# Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT",
"SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and
"OPTIONAL" in this document are to be interpreted as described in BCP 14
{{RFC2119}} {{RFC8174}} when, and only when, they appear in all capitals.

Thesis:
: A declared hypothesis recorded for later verification.

Falsifier:
: A predicate over future observable events whose firing invalidates the
  thesis. A thesis MUST have at least one falsifier when used in a
  Merkle anchor.

Causal DAG:
: A directed acyclic graph whose nodes describe variables involved in the
  thesis and whose edges describe presumed causal relationships.

Ledger:
: An append-only sequence of canonical-byte payloads, summarised by a
  Merkle root.

Causal Decay Weight:
: A scalar in the closed interval [-1, 1] representing the time-decayed
  posterior support for a thesis.

# Canonical Bytes

An FLN implementation MUST be able to produce *canonical bytes* for every
thesis. Canonical bytes are the UTF-8 encoding of a JSON text
({{RFC8259}}) produced under the following constraints:

1. The thesis object's members MUST appear in the following order:
   `id`, `domain`, `claim`, `falsifiers`, `causal_dag`, `decay`,
   `weight`, `created_at`.
2. Embedded objects (`falsifier`, `causal_node`, `causal_edge`,
   `causal_decay_params`) MUST also use declaration order.
3. JSON whitespace between tokens MUST NOT be emitted. The separators
   `","` and `":"` MUST be used as the only structural separators.
4. Numeric values MUST be emitted in the shortest representation that
   round-trips to the same IEEE-754 double. Implementations MAY rely on
   their language standard library if it satisfies this property
   (e.g., Rust `serde_json`, Python `json.dumps` with
   `separators=(",", ":")`).
5. `null` is permitted only for the `created_at` field and for the
   `deadline` field of a falsifier.

# Falsifier

```
{
  "condition": <string>,
  "deadline":  <string|null>,   // ISO 8601 date
  "triggered": <bool>
}
```

A falsifier whose `triggered` field is set to `true` MUST cause any
ledger consumer that recomputes causal-decay weights to treat the
falsifier outcome as `-1` from that point forward.

# Causal DAG

A CausalDAG SHALL contain a `nodes` array and an `edges` array. Each
node carries an `id`, a human-readable `label`, and a `kind`
∈ {`cause`, `effect`, `confounder`, `mediator`}. Each edge carries
`from`, `to`, and a `kind` ∈ {`direct`, `confounded`, `backdoor`}.

Implementations MUST reject the addition of an edge that would
introduce a cycle.

# Causal Decay

Given parameters
`{ tau_days: τ, alpha: α, regime_shift_threshold: θ }`,
the weight at time t+1 SHALL be:

~~~
w_{t+1} = w_t · exp(-Δt/τ) · (1 - I[regime_signal ≥ θ])
       + α · falsifier_outcome · (1 - exp(-Δt/τ))
~~~

where `I[·]` is the indicator function, `Δt` is the elapsed time in
days, and `regime_signal` is an externally observed regime indicator
(e.g., the VIX index for invest-domain theses). When the indicator
fires, the memory term collapses to zero; this models reflexive regime
shifts in the sense of Soros.

# Merkle Ledger

A Merkle node is the SHA-256 ({{RFC6234}}) digest of:

~~~
be64(len(payload))   || payload
be64(len(parents))   || parent_hash_1 || parent_hash_2 || ...
~~~

`be64(n)` denotes the unsigned 64-bit big-endian encoding of n.

The Merkle root of a non-empty ledger is computed by repeatedly hashing
adjacent pairs of node digests; if a layer contains an odd number of
items, the last item is duplicated. The Merkle root of an empty
ledger is undefined.

# Signing

A SignedClaim is an Ed25519 ({{RFC8032}}) signature over the canonical
bytes of a thesis. The wire format is:

~~~
{
  "payload":   <byte array>,
  "signer":    <32-byte array>,
  "signature": <64-byte array>
}
~~~

Verifiers MUST reject a SignedClaim whose `signer` is not 32 bytes or
whose `signature` is not 64 bytes.

# Security Considerations

The integrity of an FLN ledger depends on (a) the collision resistance
of SHA-256, (b) the unforgeability of Ed25519 signatures, and (c) the
secrecy of the signing key.

FLN does not provide confidentiality: canonical bytes are clear text.
Implementations that require confidentiality SHOULD encrypt the
payload at a higher layer before computing the Merkle node.

A thesis's `weight` field is advisory; it can be recomputed by any
consumer given the falsifier history and decay parameters. Consumers
SHOULD NOT trust the weight field of an unsigned thesis.

# IANA Considerations

This document has no IANA actions.

--- back

# Test Vectors

The following thesis MUST produce the canonical bytes whose SHA-256
digest equals `492448b989caf456087d7ac3a24fc1aac4c543ed496525fd2111cce874b2a574`.

~~~
id            "fixed-test"
domain        "invest"
claim         "deterministic claim"
falsifiers    [ { "condition": "x<y",
                  "deadline":  "2026-06-01",
                  "triggered": false } ]
causal_dag    nodes [
                { "id": "A", "label": "node-A", "kind": "cause" },
                { "id": "B", "label": "node-B", "kind": "effect" } ]
              edges [
                { "from": "A", "to": "B", "kind": "direct" } ]
decay         tau_days 180.0, alpha 0.1, regime_shift_threshold 30.0
weight        0.0
created_at    "2026-05-20T00:00:00Z"
~~~
