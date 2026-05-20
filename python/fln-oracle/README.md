# fln-oracle

FLN L3 — predicate evaluator backed by yfinance.

A signed FLN thesis carries human-readable falsifier `condition` strings; this
package adds a paired `*.predicates.json` file holding the machine-readable
form, and a CLI that evaluates each predicate against current market data and
writes an `*.evaluation.json` log.

The thesis bytes stay immutable (the signature still verifies); evaluations
accumulate alongside as a separate append log.

## Install

```bash
pip install -e python/fln-oracle
```

## Predicate format

```json
{
  "thesis_id": "btc-2026-q2-entry",
  "predicates": [
    {
      "falsifier_idx": 0,
      "ticker": "BTC-USD",
      "field": "close",
      "op": "lt",
      "rhs": 80000,
      "window": {"kind": "any_close", "lookback_days": 90}
    }
  ]
}
```

Supported `window.kind`:

| kind                  | meaning                                              |
| --------------------- | ---------------------------------------------------- |
| `any_close`           | True if any 1-D close in the window satisfies the op |
| `min_close`           | True if the window's minimum close satisfies the op  |
| `max_close`           | True if the window's maximum close satisfies the op  |
| `drawdown_from_high`  | True if peak-to-trough drawdown crosses `rhs`        |

## Evaluate

```bash
fln-oracle evaluate --predicates btc-q2.predicates.json --out btc-q2.evaluation.json
```

Exit code 2 ⇔ at least one falsifier triggered.
