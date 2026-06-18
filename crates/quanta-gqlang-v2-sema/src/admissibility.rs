//! E05A/B/C/D — the sema admissibility subsystem.
//!
//! The SELECT rejected-form check already lives in the parser
//! (`syntax::roots::select`); this module owns the higher-level admissibility
//! the audit found absent: the FIND subkind→capability matrix (E05A), the
//! per-root × execution-profile matrix (E05B), document-level declaration
//! semantics (E05C: duplicate names + reference cycles), and the bounded
//! execution / certified-absence expectations (E05D). All rejections are typed
//! `SemaDiagnosticV1` — never prose.

use std::collections::{BTreeMap, BTreeSet};

use quanta_gqlang_v2_syntax::roots::find::FindSubkindV1;

use quanta_gqlang_v2_syntax::roots::check::CheckConfirmIntentV1;

use crate::declarations::model_pack::{
    TypedDeclarationRootV1, TypedProfileAdmissionPolicyV1, TypedViewQueryV1,
};
use crate::interactive::sample_compare_export::TypedSampleCompareExportRootV1;
use crate::{SemaDiagnosticPageV1, SemaDiagnosticV1, TypedMetaCommandV1, TypedRootV1};

// ---------------------------------------------------------------------------
// E05A — FIND subkind → capability binding
// ---------------------------------------------------------------------------

/// The exact capability a FIND subkind binds to. The matrix is the single
/// source of truth; an unbound subkind cannot exist (compile-time length).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindAdmissibilityV1 {
    pub subkind: FindSubkindV1,
    pub required_capability: &'static str,
}

/// One row per FIND subkind — kept in lock-step with `FindSubkindV1` (7).
pub const FIND_ADMISSIBILITY_MATRIX_V1: [FindAdmissibilityV1; 7] = [
    FindAdmissibilityV1 {
        subkind: FindSubkindV1::Reachability,
        required_capability: "graph.reachability",
    },
    FindAdmissibilityV1 {
        subkind: FindSubkindV1::Flow,
        required_capability: "graph.flow",
    },
    FindAdmissibilityV1 {
        subkind: FindSubkindV1::Slice,
        required_capability: "graph.slice",
    },
    FindAdmissibilityV1 {
        subkind: FindSubkindV1::Taint,
        required_capability: "analysis.taint",
    },
    FindAdmissibilityV1 {
        subkind: FindSubkindV1::Path,
        required_capability: "graph.path",
    },
    FindAdmissibilityV1 {
        subkind: FindSubkindV1::PointsTo,
        required_capability: "analysis.points_to",
    },
    FindAdmissibilityV1 {
        subkind: FindSubkindV1::Summary,
        required_capability: "analysis.summary",
    },
];

impl FindAdmissibilityV1 {
    /// Total over `FindSubkindV1` — the matrix covers every subkind.
    #[must_use]
    pub fn for_subkind_v1(subkind: FindSubkindV1) -> &'static FindAdmissibilityV1 {
        match FIND_ADMISSIBILITY_MATRIX_V1
            .iter()
            .find(|row| row.subkind == subkind)
        {
            Some(row) => row,
            None => unreachable!("FIND_ADMISSIBILITY_MATRIX_V1 covers every FindSubkindV1"),
        }
    }
}

// ---------------------------------------------------------------------------
// E05B — per-root × execution-profile admissibility
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemaExecutionProfileV1 {
    Interactive,
    Ai,
    Lint,
    CiGate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileRootKindV1 {
    Select,
    Find,
    Check,
    Explain,
    Interactive,
    Sample,
    Declaration,
    Meta,
    Composition,
}

impl ProfileRootKindV1 {
    fn of_v1(root: &TypedRootV1) -> Self {
        match root {
            TypedRootV1::Select(_) => Self::Select,
            TypedRootV1::Find(_) => Self::Find,
            TypedRootV1::Check(_) => Self::Check,
            TypedRootV1::Explain(_) => Self::Explain,
            TypedRootV1::ShowDescribe(_) => Self::Interactive,
            TypedRootV1::SampleCompareExport(_) => Self::Sample,
            TypedRootV1::Declaration(_) => Self::Declaration,
            TypedRootV1::Meta(_) => Self::Meta,
            TypedRootV1::Composition(_) => Self::Composition,
        }
    }
}

/// A single admissibility rule: whether `root` is admissible on `profile`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionProfileRuleV1 {
    pub profile: SemaExecutionProfileV1,
    pub root: ProfileRootKindV1,
    pub admitted: bool,
    pub reason: &'static str,
}

/// The explicit REJECTION rules (Appendix J.3 spirit). Anything not listed is
/// admissible — so deterministic lanes (CI-gate / lint) fence off the
/// non-deterministic interactive/ranked roots.
pub const PROFILE_ADMISSIBILITY_MATRIX_V1: &[ExecutionProfileRuleV1] = &[
    ExecutionProfileRuleV1 {
        profile: SemaExecutionProfileV1::CiGate,
        root: ProfileRootKindV1::Sample,
        admitted: false,
        reason: "CI-gate is deterministic; ranked sampling is not admissible",
    },
    ExecutionProfileRuleV1 {
        profile: SemaExecutionProfileV1::CiGate,
        root: ProfileRootKindV1::Composition,
        admitted: false,
        reason: "CI-gate does not admit interactive composition",
    },
    ExecutionProfileRuleV1 {
        profile: SemaExecutionProfileV1::Lint,
        root: ProfileRootKindV1::Sample,
        admitted: false,
        reason: "the lint lane does not admit ranked sampling",
    },
];

impl ExecutionProfileRuleV1 {
    /// Total lookup — returns the explicit rejection rule if any, else an
    /// admitted rule.
    #[must_use]
    pub fn lookup_v1(
        root: &TypedRootV1,
        profile: SemaExecutionProfileV1,
    ) -> ExecutionProfileRuleV1 {
        let kind = ProfileRootKindV1::of_v1(root);
        match PROFILE_ADMISSIBILITY_MATRIX_V1
            .iter()
            .find(|rule| rule.profile == profile && rule.root == kind)
        {
            Some(rule) => *rule,
            None => ExecutionProfileRuleV1 {
                profile,
                root: kind,
                admitted: true,
                reason: "admissible",
            },
        }
    }
}

/// Fail-closed admission of a root on a profile.
pub fn admit_for_profile_v1(
    root: &TypedRootV1,
    profile: SemaExecutionProfileV1,
) -> Result<(), SemaDiagnosticV1> {
    let rule = ExecutionProfileRuleV1::lookup_v1(root, profile);
    if rule.admitted {
        Ok(())
    } else {
        Err(SemaDiagnosticV1 {
            code: "profile_admissibility_rejected",
            message: format!(
                "{:?} root is not admissible on the {:?} profile: {}",
                rule.root, profile, rule.reason
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// E05C — declaration semantics (gate coupling + document-level checks)
// ---------------------------------------------------------------------------

/// The admission requirements a GATE couples to a profile/pack/certification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateSemanticsV1 {
    pub requires_profile: Option<String>,
    pub requires_pack: Option<String>,
    pub requires_certified: bool,
}

fn declaration_name_v1(declaration: &TypedDeclarationRootV1) -> &str {
    match declaration {
        TypedDeclarationRootV1::View(view) => &view.name,
        TypedDeclarationRootV1::Rule(rule) => &rule.name,
        TypedDeclarationRootV1::Model(model) => &model.name,
        TypedDeclarationRootV1::Pack(pack) => &pack.name,
        TypedDeclarationRootV1::Profile(profile) => &profile.name,
        TypedDeclarationRootV1::Gate(gate) => &gate.name,
    }
}

/// The FROM table named by a VIEW's inline SELECT body (the first token after
/// `FROM`), used to build the view-reference graph.
fn inline_select_from_table_v1(sql: &str) -> Option<String> {
    let mut tokens = sql.split_whitespace();
    while let Some(token) = tokens.next() {
        if token.eq_ignore_ascii_case("FROM") {
            return tokens.next().map(str::to_string);
        }
    }
    None
}

/// Document-level admissibility (E05C): duplicate declaration names and VIEW
/// reference cycles fail closed.
pub(crate) fn admit_document_v1(roots: &[TypedRootV1]) -> Result<(), SemaDiagnosticPageV1> {
    let declarations: Vec<&TypedDeclarationRootV1> = roots
        .iter()
        .filter_map(|root| {
            if let TypedRootV1::Declaration(declaration) = root {
                Some(declaration)
            } else {
                None
            }
        })
        .collect();

    // Duplicate names across the whole document.
    let mut seen = BTreeSet::new();
    for declaration in &declarations {
        let name = declaration_name_v1(declaration);
        if !seen.insert(name.to_string()) {
            return Err(reject_v1(
                "declaration_duplicate_name",
                format!("declaration name `{name}` is declared more than once"),
            ));
        }
    }

    // VIEW reference graph → cycle.
    let view_names: BTreeSet<&str> = declarations
        .iter()
        .filter_map(|declaration| {
            if let TypedDeclarationRootV1::View(view) = declaration {
                Some(view.name.as_str())
            } else {
                None
            }
        })
        .collect();
    let mut edges: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for declaration in &declarations {
        if let TypedDeclarationRootV1::View(view) = declaration {
            if let TypedViewQueryV1::InlineSelect(sql) = &view.query {
                if let Some(table) = inline_select_from_table_v1(sql) {
                    if view_names.contains(table.as_str()) {
                        edges.entry(view.name.as_str()).or_default().push(table);
                    }
                }
            }
        }
    }
    if let Some(cycle_node) = first_cycle_node_v1(&edges) {
        return Err(reject_v1(
            "declaration_cycle",
            format!("VIEW `{cycle_node}` participates in a reference cycle"),
        ));
    }

    enforce_trust_policy_v1(roots)?;
    Ok(())
}

/// E10B/F07 — when the document declares `REQUIRE CERTIFIED`, the trust policy is
/// active and fails closed on: ranked sampling (candidate-set, not certified),
/// a CHECK whose semantic confirm is not required, and a PROFILE whose admission
/// is weaker than certified-only. A document WITHOUT `REQUIRE CERTIFIED` is
/// untouched (no over-blocking).
pub fn enforce_trust_policy_v1(roots: &[TypedRootV1]) -> Result<(), SemaDiagnosticPageV1> {
    let require_certified = roots.iter().any(|root| {
        matches!(
            root,
            TypedRootV1::Meta(TypedMetaCommandV1::RequireCertified { .. })
        )
    });
    if !require_certified {
        return Ok(());
    }

    // Guarded `match` (explicit over all 9 root variants — no wildcard) so a new
    // root variant forces a compile decision here rather than being silently
    // swallowed. The trailing OR arm is the "admitted" fall-through for roots a
    // guard rejected on (e.g. a confirmed CHECK, a non-Sample interactive root).
    for root in roots {
        match root {
            TypedRootV1::SampleCompareExport(TypedSampleCompareExportRootV1::Sample { .. }) => {
                return Err(reject_v1(
                    "require_certified_rejects_sampling",
                    "REQUIRE CERTIFIED forbids ranked/candidate sampling (not certified truth)"
                        .to_string(),
                ));
            }
            TypedRootV1::Check(check)
                if check
                    .intent()
                    .is_ok_and(|intent| intent.confirm != CheckConfirmIntentV1::Require) =>
            {
                return Err(reject_v1(
                    "require_certified_rejects_unconfirmed_check",
                    "REQUIRE CERTIFIED requires CHECK semantic confirm=require".to_string(),
                ));
            }
            TypedRootV1::Declaration(TypedDeclarationRootV1::Profile(profile))
                if profile.admission != TypedProfileAdmissionPolicyV1::CertifiedOnly =>
            {
                return Err(reject_v1(
                    "require_certified_profile_incompatible",
                    format!(
                        "REQUIRE CERTIFIED requires certified-only PROFILE admission; \
                         `{}` declares {:?}",
                        profile.name, profile.admission
                    ),
                ));
            }
            TypedRootV1::Select(_)
            | TypedRootV1::Find(_)
            | TypedRootV1::Check(_)
            | TypedRootV1::Explain(_)
            | TypedRootV1::ShowDescribe(_)
            | TypedRootV1::SampleCompareExport(_)
            | TypedRootV1::Declaration(_)
            | TypedRootV1::Meta(_)
            | TypedRootV1::Composition(_) => {}
        }
    }
    Ok(())
}

fn first_cycle_node_v1(edges: &BTreeMap<&str, Vec<String>>) -> Option<String> {
    // DFS with a recursion stack; returns the first node found on a back-edge.
    fn visit<'a>(
        node: &'a str,
        edges: &'a BTreeMap<&'a str, Vec<String>>,
        on_stack: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> Option<String> {
        if done.contains(node) {
            return None;
        }
        if !on_stack.insert(node.to_string()) {
            return Some(node.to_string());
        }
        if let Some(targets) = edges.get(node) {
            for target in targets {
                if let Some(found) = visit(target, edges, on_stack, done) {
                    return Some(found);
                }
            }
        }
        on_stack.remove(node);
        done.insert(node.to_string());
        None
    }

    let mut on_stack = BTreeSet::new();
    let mut done = BTreeSet::new();
    for node in edges.keys() {
        if let Some(found) = visit(node, edges, &mut on_stack, &mut done) {
            return Some(found);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// E05D — bounded execution / certified absence
// ---------------------------------------------------------------------------

/// A certified-absence expectation: an absence claim is only admissible when it
/// is certified (never a heuristic guess).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertifiedAbsenceV1 {
    pub claim: String,
    pub certified: bool,
}

impl CertifiedAbsenceV1 {
    /// Fail-closed: an uncertified absence claim is rejected.
    pub fn admit_v1(claim: impl Into<String>, certified: bool) -> Result<Self, SemaDiagnosticV1> {
        let claim = claim.into();
        if !certified {
            return Err(SemaDiagnosticV1 {
                code: "uncertified_absence_rejected",
                message: format!("absence claim `{claim}` is not certified and cannot be admitted"),
            });
        }
        Ok(Self { claim, certified })
    }
}

/// The outcome of a bounded execution under a budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetOutcomeV1 {
    /// Budget exhausted before a sound answer — fail closed.
    RejectedBudget,
    /// Bounded but with an explicit degraded reason surfaced.
    Degraded,
}

impl BudgetOutcomeV1 {
    #[must_use]
    pub const fn is_fail_closed_v1(self) -> bool {
        matches!(self, Self::RejectedBudget)
    }
}

fn reject_v1(code: &'static str, message: String) -> SemaDiagnosticPageV1 {
    SemaDiagnosticPageV1 {
        diagnostics: vec![SemaDiagnosticV1 { code, message }],
    }
}
