//! `LET` / `WITH` composition typing (E03C ticket 02).
//!
//! Resolves the `WITH` alias to a binding *index* (so the IR never re-resolves
//! strings), enforces the D3 producer allow-set, and reuses the per-root
//! analyzers verbatim for producers and the body — composition adds structure,
//! never re-implements root semantics. All rejects are fail-closed with a
//! stable `composition_*` code; there is no default arm and no heuristic.

use crate::{analyze_root_v1, SemaDiagnosticPageV1, TypedRootV1};
use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::interactive::composition::CompositionRootAstV1;

/// The working-set basis a `LET` producer establishes (D3 total mapping).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedWorkingSetBasisV1 {
    SearchQuery,
    AiDiscovery,
}

/// Profile gating carrier (D1). Exactly one variant today; reintroducing
/// profile admission later is an enum-variant addition, not a retrofit. The
/// absence of gating is itself typed — never a "default allow" string.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompositionGatingV1 {
    NotYetGated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedLetBindingV1 {
    pub name: String,
    pub producer: Box<TypedRootV1>,
    pub basis: TypedWorkingSetBasisV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCompositionV1 {
    pub bindings: Vec<TypedLetBindingV1>,
    /// Index into `bindings` the `WITH` resolved to (never a name).
    pub with_index: usize,
    pub body: Box<TypedRootV1>,
    pub gating: CompositionGatingV1,
}

pub fn analyze_composition_v1(
    spec: &CompositionRootAstV1,
) -> Result<TypedCompositionV1, SemaDiagnosticPageV1> {
    let mut bindings = Vec::with_capacity(spec.lets.len());
    for binding in &spec.lets {
        // Producer admissibility is checked before analysis so an unsupported
        // producer fails as `composition_invalid_producer` rather than leaking
        // an unrelated analyzer error.
        let basis = composition_basis_for_root_v1(&binding.producer)?;
        let producer = analyze_root_v1(&binding.producer)?;
        bindings.push(TypedLetBindingV1 {
            name: binding.name.clone(),
            producer: Box::new(producer),
            basis,
        });
    }

    let with_index = spec
        .lets
        .iter()
        .position(|binding| binding.name == spec.with_name)
        .ok_or_else(|| {
            composition_page_v1(format!(
                "composition_undefined_binding: WITH `{}` names no LET binding",
                spec.with_name
            ))
        })?;

    // The body must be a working-set *consumer*: `WITH <set>` scopes the body
    // to the bound working set, so a body that cannot operate over a candidate
    // set (e.g. `SHOW`) would make the binding meaningless. Checked before
    // analysis so a non-consumer body fails as `composition_invalid_body`.
    composition_body_consumer_v1(&spec.body)?;
    let body = analyze_root_v1(&spec.body)?;
    Ok(TypedCompositionV1 {
        bindings,
        with_index,
        body: Box::new(body),
        gating: CompositionGatingV1::NotYetGated,
    })
}

/// The roots that meaningfully consume a working set (the analysis/query plane:
/// `FIND`, `SELECT`, `CHECK`, `EXPLAIN`). `WITH <set>` scopes the body to the
/// bound working set, so only a consumer is an admissible body. Total over
/// `AstRootV1` (no default arm) — a new root family forces a deliberate
/// consumer/non-consumer decision here. This is the asymmetric complement of
/// the producer allow-set: producers create working sets, consumers narrow over
/// them, and the two sets are intentionally different.
fn composition_body_consumer_v1(root: &AstRootV1) -> Result<(), SemaDiagnosticPageV1> {
    match root {
        AstRootV1::Find(_) | AstRootV1::Select(_) | AstRootV1::Check(_) | AstRootV1::Explain(_) => {
            Ok(())
        }
        AstRootV1::Sample(_)
        | AstRootV1::Show(_)
        | AstRootV1::Describe(_)
        | AstRootV1::Compare(_)
        | AstRootV1::Export(_)
        | AstRootV1::View(_)
        | AstRootV1::Rule(_)
        | AstRootV1::Model(_)
        | AstRootV1::Pack(_)
        | AstRootV1::Profile(_)
        | AstRootV1::Gate(_)
        | AstRootV1::Meta(_)
        | AstRootV1::Composition(_) => Err(composition_page_v1(
            "composition_invalid_body: WITH body must be a working-set consumer \
             (FIND, SELECT, CHECK, or EXPLAIN)"
                .to_string(),
        )),
    }
}

/// D3 producer allow-set, total over `AstRootV1` (no default arm) so a new root
/// family forces a deliberate decision here instead of silently being accepted
/// or rejected as a working-set producer.
fn composition_basis_for_root_v1(
    root: &AstRootV1,
) -> Result<TypedWorkingSetBasisV1, SemaDiagnosticPageV1> {
    match root {
        AstRootV1::Sample(_) | AstRootV1::Select(_) => Ok(TypedWorkingSetBasisV1::SearchQuery),
        AstRootV1::Find(_) => Ok(TypedWorkingSetBasisV1::AiDiscovery),
        AstRootV1::Check(_)
        | AstRootV1::Explain(_)
        | AstRootV1::Show(_)
        | AstRootV1::Describe(_)
        | AstRootV1::Compare(_)
        | AstRootV1::Export(_)
        | AstRootV1::View(_)
        | AstRootV1::Rule(_)
        | AstRootV1::Model(_)
        | AstRootV1::Pack(_)
        | AstRootV1::Profile(_)
        | AstRootV1::Gate(_)
        | AstRootV1::Meta(_)
        | AstRootV1::Composition(_) => Err(composition_page_v1(
            "composition_invalid_producer: LET producer must be SAMPLE, SELECT, or FIND"
                .to_string(),
        )),
    }
}

fn composition_page_v1(message: String) -> SemaDiagnosticPageV1 {
    SemaDiagnosticPageV1 {
        diagnostics: vec![message],
    }
}
