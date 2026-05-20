"""FLN L3 — Falsifier evaluator.

Reads ``*.predicates.json`` paired with a ``*.thesis.json`` and decides which
falsifiers have triggered against a market-data source. Writes a
``*.evaluation.json`` log entry that can be appended to the FLN ledger as a
*new* MerkleNode whose parents include the original thesis hash.
"""

from .evaluator import EvaluationResult, evaluate_predicate, evaluate_predicates
from .predicate import Predicate, PredicateSet, Window
from .sources import InMemorySource, MarketSource, YFinanceSource

__version__ = "0.1.0"

__all__ = [
    "EvaluationResult",
    "evaluate_predicate",
    "evaluate_predicates",
    "Predicate",
    "PredicateSet",
    "Window",
    "InMemorySource",
    "MarketSource",
    "YFinanceSource",
]
