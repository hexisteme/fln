"""Canonical-bytes strictness validator — Python mirror of ``fln_core::canonical``.

Wire-compatible with the Rust validator: same rejection rules, same error
shape (as a single :class:`CanonicalError` with a message tag).
"""

from __future__ import annotations

import json
import re
import unicodedata
from typing import Any

_ISO8601_STRICT = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?Z$"
)
_TIMESTAMP_FIELDS = {"created_at", "anchored_at", "deadline"}


class CanonicalError(ValueError):
    """Raised when canonical bytes fail strictness checks."""


def is_strict_iso8601_utc(s: str) -> bool:
    return bool(_ISO8601_STRICT.fullmatch(s))


def _is_nfc(s: str) -> bool:
    return unicodedata.normalize("NFC", s) == s


def _detect_duplicate_keys_pair_hook(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    seen: set[str] = set()
    for key, _ in pairs:
        if key in seen:
            raise CanonicalError(f"duplicate key `{key}` in canonical bytes")
        seen.add(key)
    return dict(pairs)


def _walk(value: Any, path: str, parent_is_timestamp: bool) -> None:
    if isinstance(value, str):
        if not _is_nfc(value):
            raise CanonicalError(
                f"string field `{path}` is not in Unicode Normalization Form C"
            )
        if parent_is_timestamp and not is_strict_iso8601_utc(value):
            raise CanonicalError(
                f"string field `{path}` violates ISO 8601 strict UTC format: {value}"
            )
    elif isinstance(value, dict):
        for k, v in value.items():
            if not _is_nfc(k):
                raise CanonicalError(
                    f"object key `{path}.{k}(key)` is not in Unicode Normalization Form C"
                )
            child = f"{path}.{k}" if path else k
            _walk(v, child, k in _TIMESTAMP_FIELDS)
    elif isinstance(value, list):
        for i, v in enumerate(value):
            _walk(v, f"{path}[{i}]", False)


def validate_canonical_bytes(payload: bytes) -> None:
    """Reject duplicate keys, non-NFC strings, and loose timestamps."""
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError as e:
        raise CanonicalError(f"payload is not valid UTF-8: {e}") from e
    try:
        parsed = json.loads(text, object_pairs_hook=_detect_duplicate_keys_pair_hook)
    except json.JSONDecodeError as e:
        raise CanonicalError(f"JSON parse error: {e}") from e
    _walk(parsed, "", False)


__all__ = ["CanonicalError", "is_strict_iso8601_utc", "validate_canonical_bytes"]
