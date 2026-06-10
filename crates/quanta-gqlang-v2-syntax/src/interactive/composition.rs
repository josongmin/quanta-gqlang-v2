//! `LET` / `WITH` working-set composition AST (E03C).
//!
//! `LET <name> = <producer-root>` and `WITH <name>` are composition operators,
//! not roots: the parser groups a `[LET]* WITH <body>` run into a single
//! [`CompositionRootAstV1`]. Binding names are deterministic identifiers (no
//! heuristic repair); grammar enforcement lives in `parser.rs` (it owns the
//! diagnostic helper), this module owns the typed shapes + name rule.

use crate::ast::AstRootV1;

/// One `LET <name> = <producer>` binding. `producer` is a fully parsed root,
/// never a raw string — the producer's own grammar is reused verbatim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetBindingAstV1 {
    pub name: String,
    pub producer: Box<AstRootV1>,
}

/// A grouped composition: zero or more bindings, the `WITH` narrowing alias,
/// and the body root that follows it. Alias resolution against the bindings is
/// a sema concern (an empty-`lets` composition is structurally valid here and
/// fails closed downstream as `composition_undefined_binding`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionRootAstV1 {
    pub lets: Vec<LetBindingAstV1>,
    pub with_name: String,
    pub body: Box<AstRootV1>,
}

/// D4 binding-name shape: `[a-z][a-z0-9_]*`. Deterministic, no Unicode
/// case-folding, no trimming beyond what the caller already did.
#[must_use]
pub fn is_valid_binding_name_v1(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}
