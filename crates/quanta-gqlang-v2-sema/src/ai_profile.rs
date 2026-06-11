//! E05E — the AI exploration lane admissibility (sema half).
//!
//! The AI lane is never a final-truth oracle: it proposes candidate sets it
//! cannot itself certify, so every root it admits carries a reconfirmation
//! discipline and any root that demands final/certified policy truth fails
//! closed. This is the sema mirror of the IR `planning_ai_final_truth_forbidden`
//! fence, and it routes its structural base through F02's profile matrix
//! (`SemaExecutionProfileV1::Ai`) so the two admission planes share one source.

use quanta_gqlang_v2_syntax::roots::check::CheckConfirmIntentV1;

use crate::admissibility::{admit_for_profile_v1, SemaExecutionProfileV1};
use crate::{SemaDiagnosticV1, TypedMetaCommandV1, TypedRootV1};

/// The AI-lane admission outcome. The exploration lane never asserts final truth,
/// so an admitted root is always reconfirmation-required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedAiProfileV1 {
    pub reconfirmation_required: bool,
}

/// Admit `root` onto the AI exploration lane (E05E). Defers to F02's profile
/// matrix for the structural base, then fails closed on roots that demand final
/// or certified policy truth — mirroring the IR `planning_ai_final_truth_forbidden`
/// fence. Every admitted root is reconfirmation-required.
pub fn ai_lane_admissibility_v1(root: &TypedRootV1) -> Result<TypedAiProfileV1, SemaDiagnosticV1> {
    admit_for_profile_v1(root, SemaExecutionProfileV1::Ai)?;

    if demands_final_policy_truth_v1(root) {
        return Err(SemaDiagnosticV1 {
            code: "ai_lane_final_truth_forbidden",
            message: "the AI exploration lane cannot assert final/certified policy truth; \
                      it proposes candidate sets that require reconfirmation"
                .to_string(),
        });
    }

    Ok(TypedAiProfileV1 {
        reconfirmation_required: true,
    })
}

/// Whether a typed root demands final policy truth, and so cannot ride the
/// reconfirmation-only AI exploration lane. `if let` (not a wildcard `match`) so
/// a new root variant defaults to admissible, never silently swallowed.
fn demands_final_policy_truth_v1(root: &TypedRootV1) -> bool {
    if let TypedRootV1::Meta(TypedMetaCommandV1::RequireCertified { .. }) = root {
        // REQUIRE CERTIFIED demands certified final truth.
        return true;
    }
    if let TypedRootV1::Check(check) = root {
        // A require-confirmed CHECK asserts an authoritative policy verdict.
        return check
            .intent()
            .is_ok_and(|intent| intent.confirm == CheckConfirmIntentV1::Require);
    }
    false
}
