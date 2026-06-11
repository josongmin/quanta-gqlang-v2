//! E07B — retrieval fragment dialect policy.
//!
//! A retrieval `fragment:<dialect>:<text>` lets a query embed a host-search
//! dialect fragment. The policy restricts which dialects are admissible so an
//! unsupported dialect (one the host search surface cannot honor) fails closed
//! rather than being passed through opaquely.

use crate::SemaDiagnosticV1;

/// The closed set of admissible retrieval fragment dialects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FragmentPolicyV1;

impl FragmentPolicyV1 {
    /// Dialects the host search surface can honor.
    pub const ALLOWED_DIALECTS_V1: &'static [&'static str] = &["sourcegraph"];

    #[must_use]
    pub fn allows_v1(dialect: &str) -> bool {
        Self::ALLOWED_DIALECTS_V1
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(dialect))
    }

    /// Fail-closed admission of a fragment dialect.
    pub fn admit_dialect_v1(dialect: &str) -> Result<(), SemaDiagnosticV1> {
        if dialect.trim().is_empty() {
            return Err(SemaDiagnosticV1 {
                code: "fragment_dialect_missing",
                message: "retrieval fragment requires a dialect (`fragment:<dialect>:<text>`)"
                    .to_string(),
            });
        }
        if Self::allows_v1(dialect) {
            Ok(())
        } else {
            Err(SemaDiagnosticV1 {
                code: "fragment_dialect_rejected",
                message: format!(
                    "retrieval fragment dialect `{dialect}` is not admissible; allowed: {:?}",
                    Self::ALLOWED_DIALECTS_V1
                ),
            })
        }
    }
}
