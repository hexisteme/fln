"""Cross-language strictness tests — Python mirror of crates/fln-core/src/canonical.rs."""

from __future__ import annotations

import pytest

from fln import CanonicalError, is_strict_iso8601_utc, validate_canonical_bytes


def test_happy_path_passes():
    validate_canonical_bytes(
        b'{"version":1,"claim":"hello","created_at":"2026-05-21T12:00:00Z"}'
    )


def test_fractional_seconds_pass():
    validate_canonical_bytes(b'{"anchored_at":"2026-05-21T12:00:00.123Z"}')


def test_duplicate_keys_rejected():
    with pytest.raises(CanonicalError, match="duplicate"):
        validate_canonical_bytes(b'{"version":1,"version":2}')


def test_nfd_string_rejected():
    payload = '{"claim":"caf' + "é" + '"}'  # NFD form of "café"
    with pytest.raises(CanonicalError, match="Normalization"):
        validate_canonical_bytes(payload.encode("utf-8"))


def test_nfc_string_passes():
    validate_canonical_bytes('{"claim":"café"}'.encode("utf-8"))


def test_timestamp_with_offset_rejected():
    with pytest.raises(CanonicalError, match="ISO 8601"):
        validate_canonical_bytes(b'{"created_at":"2026-05-21T12:00:00+02:00"}')


def test_timestamp_without_seconds_rejected():
    with pytest.raises(CanonicalError, match="ISO 8601"):
        validate_canonical_bytes(b'{"created_at":"2026-05-21T12:00Z"}')


def test_is_strict_iso8601():
    assert is_strict_iso8601_utc("2026-05-21T12:00:00Z")
    assert is_strict_iso8601_utc("2026-05-21T12:00:00.123456789Z")
    assert not is_strict_iso8601_utc("2026-05-21T12:00Z")
    assert not is_strict_iso8601_utc("2026-05-21T12:00:00+00:00")
    assert not is_strict_iso8601_utc("2026-05-21T12:00:00.1234567890Z")  # 10 digits


def test_canonical_validates_real_thesis():
    from fln import Domain, Thesis

    t = Thesis.new("ok-id", Domain.INVEST, "ASCII claim")
    t.created_at = "2026-05-21T00:00:00Z"
    validate_canonical_bytes(t.canonical_bytes())


def test_nonce_is_wire_additive():
    from fln import Domain, Thesis

    plain = Thesis.new("id", Domain.INVEST, "x")
    with_nonce = Thesis.new("id", Domain.INVEST, "x")
    with_nonce.nonce = "deadbeefdeadbeefdeadbeefdeadbeef"

    plain_bytes = plain.canonical_bytes()
    nonced_bytes = with_nonce.canonical_bytes()

    assert b'"nonce"' not in plain_bytes  # absent when None
    assert b'"nonce":"deadbeef' in nonced_bytes
    assert plain_bytes != nonced_bytes
