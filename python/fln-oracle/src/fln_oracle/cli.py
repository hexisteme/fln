"""fln-oracle CLI — evaluate falsifier predicates against market data."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .evaluator import evaluate_predicates
from .predicate import PredicateSet
from .sources import YFinanceSource


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="fln-oracle")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_eval = sub.add_parser("evaluate", help="Evaluate predicates against yfinance")
    p_eval.add_argument("--predicates", required=True, type=Path)
    p_eval.add_argument("--out", required=True, type=Path)

    args = parser.parse_args(argv)

    if args.cmd == "evaluate":
        ps = PredicateSet.load(args.predicates)
        source = YFinanceSource()
        report = evaluate_predicates(ps, source)
        args.out.write_text(json.dumps(report, indent=2))
        print(f"wrote {args.out}")
        for r in report["results"]:
            mark = "🔥" if r["triggered"] else "·"
            val = f"{r['observed_value']:.4f}" if r["observed_value"] is not None else "-"
            print(f"  {mark} idx={r['falsifier_idx']} {r['ticker']:<10} observed={val:<12} {r['reason']}")
        if report["any_triggered"]:
            print("at least one falsifier TRIGGERED")
            return 2
        print("no falsifiers triggered")
        return 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
