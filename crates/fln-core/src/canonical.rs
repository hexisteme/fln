//! Canonical-bytes strictness validator.
//!
//! v0.2.1 closes three malleability vectors flagged by the post-v0.2 audit:
//!
//! 1. **Duplicate JSON keys** — serde_json silently keeps the last; an
//!    attacker who crafts ``{"version":1,"version":2,...}`` sees Rust pick
//!    `2` while a different parser might pick `1`. We reject any JSON
//!    object with duplicate keys.
//! 2. **UTF-8 normalization malleability (NFC vs NFD)** — the same visual
//!    string can be encoded as `é` (NFC `é`) or `é` (NFD).
//!    Both verify under SHA-256 but render identically; downstream
//!    semantics may diverge. We require every string in canonical bytes
//!    to already be in NFC.
//! 3. **ISO 8601 looseness** — `2026-05-21T00:00:00Z` and
//!    `2026-05-21T02:00:00+02:00` describe the same instant but hash
//!    differently. We constrain timestamps to a strict UTC form.

use serde_json::Value;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CanonicalError {
    #[error("JSON parse error: {0}")]
    Json(String),
    #[error("duplicate key `{0}` in canonical bytes")]
    DuplicateKey(String),
    #[error("string field `{0}` is not in Unicode Normalization Form C")]
    NonNfc(String),
    #[error("string field `{field}` violates ISO 8601 strict UTC format: {value}")]
    BadTimestamp { field: String, value: String },
    #[error("re-emitted canonical bytes differ from input — malleability vector")]
    NotRoundTrip,
}

/// Strict ISO 8601 UTC: ``YYYY-MM-DDTHH:MM:SS(.f{1,9})?Z``.
pub fn is_strict_iso8601_utc(s: &str) -> bool {
    let bytes = s.as_bytes();
    // Minimum length: 20 (YYYY-MM-DDTHH:MM:SSZ)
    if bytes.len() < 20 {
        return false;
    }
    // Date: 4-2-2
    if !(bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(|b| b.is_ascii_digit()))
    {
        return false;
    }
    if bytes[10] != b'T' {
        return false;
    }
    // Time: 2:2:2
    if !(bytes[11..13].iter().all(|b| b.is_ascii_digit())
        && bytes[13] == b':'
        && bytes[14..16].iter().all(|b| b.is_ascii_digit())
        && bytes[16] == b':'
        && bytes[17..19].iter().all(|b| b.is_ascii_digit()))
    {
        return false;
    }
    // Tail: 'Z' OR '.<digits>Z'
    match &bytes[19..] {
        [b'Z'] => true,
        [b'.', rest @ .., b'Z'] => !rest.is_empty() && rest.len() <= 9 && rest.iter().all(|b| b.is_ascii_digit()),
        _ => false,
    }
}

/// Re-parses `bytes` as JSON, walks the tree to check for duplicate keys,
/// validates NFC normalization on all strings, and (for fields named
/// `created_at`, `anchored_at`, `deadline`) enforces the strict ISO 8601
/// subset.
pub fn validate_canonical_bytes(bytes: &[u8]) -> Result<(), CanonicalError> {
    let raw = std::str::from_utf8(bytes).map_err(|e| CanonicalError::Json(e.to_string()))?;
    // Walk the JSON token stream once to detect duplicate keys — serde_json
    // would silently dedupe them.
    detect_duplicate_keys(raw)?;
    // After we know keys are unique, parse into Value and walk for content
    // validation (NFC + timestamp strictness).
    let v: Value = serde_json::from_str(raw).map_err(|e| CanonicalError::Json(e.to_string()))?;
    walk_value(&v, "", false)?;
    Ok(())
}

fn detect_duplicate_keys(raw: &str) -> Result<(), CanonicalError> {
    use std::collections::HashSet;
    let mut stack: Vec<HashSet<String>> = Vec::new();
    let mut chars = raw.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '{' => stack.push(HashSet::new()),
            '}' => {
                stack.pop();
            }
            '"' => {
                // We only care about keys: a string immediately followed by ':'
                let (end, _) = scan_string_end(raw, i + 1)?;
                let key = unescape_json_string(&raw[i + 1..end]);
                // Consume the closing quote
                while let Some(&(j, _)) = chars.peek() {
                    if j <= end {
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Is the next non-whitespace char ':'?
                let mut peek = chars.clone();
                let mut is_key = false;
                while let Some(&(_, p)) = peek.peek() {
                    if p.is_whitespace() {
                        peek.next();
                    } else {
                        is_key = p == ':';
                        break;
                    }
                }
                if is_key
                    && let Some(scope) = stack.last_mut()
                    && !scope.insert(key.clone())
                {
                    return Err(CanonicalError::DuplicateKey(key));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn scan_string_end(raw: &str, start: usize) -> Result<(usize, ()), CanonicalError> {
    let bytes = raw.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Ok((i, ())),
            _ => i += 1,
        }
    }
    Err(CanonicalError::Json("unterminated string".into()))
}

fn unescape_json_string(s: &str) -> String {
    // Coarse unescape — sufficient for duplicate-key tracking.
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(match next {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    other => other,
                });
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn is_nfc(s: &str) -> bool {
    // Always do the full canonicalize-and-compare; `is_nfc_quick` returns
    // Yes/No/Maybe and Maybe values must still be confirmed.
    let canon: String = s.chars().nfc().collect();
    canon == s
}

fn walk_value(value: &Value, path: &str, parent_is_timestamp_field: bool) -> Result<(), CanonicalError> {
    match value {
        Value::String(s) => {
            if !is_nfc(s) {
                return Err(CanonicalError::NonNfc(path.to_string()));
            }
            if parent_is_timestamp_field && !is_strict_iso8601_utc(s) {
                return Err(CanonicalError::BadTimestamp {
                    field: path.to_string(),
                    value: s.clone(),
                });
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                if !is_nfc(k) {
                    return Err(CanonicalError::NonNfc(format!("{path}.{k}(key)")));
                }
                let child_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                let is_ts = matches!(k.as_str(), "created_at" | "anchored_at" | "deadline");
                walk_value(v, &child_path, is_ts)?;
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let child_path = format!("{path}[{i}]");
                walk_value(v, &child_path, false)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_passes() {
        let bytes = br#"{"version":1,"claim":"hello","created_at":"2026-05-21T12:00:00Z"}"#;
        validate_canonical_bytes(bytes).unwrap();
    }

    #[test]
    fn happy_path_with_fractional_seconds() {
        let bytes = br#"{"claim":"ok","anchored_at":"2026-05-21T12:00:00.123Z"}"#;
        validate_canonical_bytes(bytes).unwrap();
    }

    #[test]
    fn duplicate_keys_are_rejected() {
        let bytes = br#"{"version":1,"version":2}"#;
        let err = validate_canonical_bytes(bytes).unwrap_err();
        assert!(matches!(err, CanonicalError::DuplicateKey(ref k) if k == "version"));
    }

    #[test]
    fn nfd_string_is_rejected() {
        // "é" decomposed as e + combining-acute (NFD form).
        let payload = format!(r#"{{"claim":"caf{}"}}"#, "e\u{0301}");
        let err = validate_canonical_bytes(payload.as_bytes()).unwrap_err();
        assert!(matches!(err, CanonicalError::NonNfc(_)), "{err:?}");
    }

    #[test]
    fn nfc_string_passes() {
        let payload = r#"{"claim":"café"}"#;
        validate_canonical_bytes(payload.as_bytes()).unwrap();
    }

    #[test]
    fn timestamp_with_offset_is_rejected() {
        let bytes = br#"{"created_at":"2026-05-21T12:00:00+02:00"}"#;
        let err = validate_canonical_bytes(bytes).unwrap_err();
        assert!(matches!(err, CanonicalError::BadTimestamp { .. }));
    }

    #[test]
    fn timestamp_without_seconds_is_rejected() {
        let bytes = br#"{"created_at":"2026-05-21T12:00Z"}"#;
        assert!(validate_canonical_bytes(bytes).is_err());
    }

    #[test]
    fn is_strict_iso8601_examples() {
        assert!(is_strict_iso8601_utc("2026-05-21T12:00:00Z"));
        assert!(is_strict_iso8601_utc("2026-05-21T12:00:00.000Z"));
        assert!(is_strict_iso8601_utc("2026-05-21T12:00:00.123456789Z"));
        assert!(!is_strict_iso8601_utc("2026-05-21 12:00:00Z"));
        assert!(!is_strict_iso8601_utc("2026-05-21T12:00:00+00:00"));
        assert!(!is_strict_iso8601_utc("2026-05-21T12:00:00.1234567890Z")); // 10 fractional digits
        assert!(!is_strict_iso8601_utc("2026-05-21T12:00Z"));
    }
}
