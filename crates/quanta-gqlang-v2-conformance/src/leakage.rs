//! Single source of truth for the runtime-truth token denylist used by the
//! CLI and LSP "no runtime leakage" sweeps.
//!
//! Before this module each sweep carried its own hand-maintained denylist (the
//! CLI one a superset of the LSP one) and matched with a plain case-sensitive
//! `contains`, which (a) let the two lists drift and (b) false-positived on
//! honest identifiers that merely *contain* a token as a substring
//! (`query_result_count`, `no_result_reason`). This module owns the one list and
//! a boundary-aware, case-insensitive matcher both sweeps call.
//!
//! NOTE: this file legitimately enumerates the denylist tokens as DATA, so it is
//! explicitly exempted from the architecture checker's runtime-truth source
//! scan (see `RUNTIME_TRUTH_SCAN_EXEMPT_FILES` in `check_architecture.py`).

/// Every token that must never appear as a standalone runtime-truth claim in a
/// read-only front door's output. Tokens ending in `=` are key-prefix shapes
/// (`rows=5`, `verdict=true`); the rest are identifier/JSON-key shapes.
pub const RUNTIME_TRUTH_DENYLIST_V1: &[&str] = &[
    "result_count",
    "row_count",
    "rows_returned",
    "rows=",
    "match_count",
    "matches_found",
    "results_found",
    "num_results",
    "execution_time",
    "elapsed_ms",
    "executed_at",
    "ran_at",
    "provider_response",
    "provider_endpoint",
    "routed_to",
    "host_dto",
    "host_response",
    "backend_result",
    "edit_applied",
    "applied_edit",
    "workspace_edit_applied",
    "materialized_result",
    "result_materialized",
    "runtime_truth",
    "confirmed_true",
    "verdict=",
    "actual_result",
];

const fn is_word_byte_v1(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Return the first denylist token that appears in `haystack` as a *standalone*
/// token — case-insensitive, and bounded so a token does not match when it is
/// merely a substring inside a larger identifier.
///
/// Boundary rule: an occurrence counts only if the byte before it is not a word
/// byte (`[A-Za-z0-9_]`), and — when the token's last char is itself a word byte
/// — the byte after it is not a word byte either. So `result_count` matches in
/// `result_count=3` but NOT inside `query_result_count`; `rows=` (own trailing
/// boundary) matches `rows=5` but NOT `table_rows=5`.
#[must_use]
pub fn find_runtime_truth_token_v1(haystack: &str) -> Option<&'static str> {
    let lower = haystack.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for &token in RUNTIME_TRUTH_DENYLIST_V1 {
        let Some(&last_byte) = token.as_bytes().last() else {
            continue;
        };
        let last_is_word = is_word_byte_v1(last_byte);
        let mut from = 0usize;
        while let Some(rel) = lower.get(from..).and_then(|tail| tail.find(token)) {
            let start = from + rel;
            let end = start + token.len();
            let before_ok = start
                .checked_sub(1)
                .and_then(|prev| bytes.get(prev))
                .map_or(true, |&byte| !is_word_byte_v1(byte));
            let after_ok =
                !last_is_word || bytes.get(end).map_or(true, |&byte| !is_word_byte_v1(byte));
            if before_ok && after_ok {
                return Some(token);
            }
            from = start + 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_standalone_runtime_truth_token_v1() {
        assert_eq!(
            find_runtime_truth_token_v1("query.result_count=3"),
            Some("result_count")
        );
        assert_eq!(
            find_runtime_truth_token_v1("status rows=5 done"),
            Some("rows=")
        );
    }

    #[test]
    fn is_case_insensitive_v1() {
        assert_eq!(
            find_runtime_truth_token_v1("RESULT_COUNT: 9"),
            Some("result_count")
        );
    }

    #[test]
    fn does_not_false_positive_inside_a_larger_identifier_v1() {
        // The substring `result_count` lives inside this honest identifier, but it
        // is not a standalone token, so the plain-`contains` false positive the
        // old sweeps risked is gone.
        assert_eq!(find_runtime_truth_token_v1("query_result_count"), None);
        assert_eq!(find_runtime_truth_token_v1("no_result_reason"), None);
        assert_eq!(find_runtime_truth_token_v1("table_rows=5"), None);
    }

    #[test]
    fn clean_output_has_no_token_v1() {
        assert_eq!(
            find_runtime_truth_token_v1("explain.degraded[0].code=routing-unavailable"),
            None
        );
    }
}
