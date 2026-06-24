//! Parameterized signatures / interfaces for reusable query abstractions
//! (deliverable 2).
//!
//! A [`SignatureV1`] is the reuse boundary's declared interface: a name, an
//! ordered list of typed parameters, and a result obligation. It lets a module
//! publish a reusable query abstraction whose callers bind concrete arguments
//! by position. This crate owns the STRUCTURAL contract (arity + labeled
//! parameter identity) and the deterministic instantiation check; closed type
//! legality of the bound arguments remains ticket 03's authority (`D13`), so the
//! parameter/argument "type" here is an opaque symbolic tag, not a resolved
//! [`quanta_gqlang_v2_hir`] type.

use crate::diagnostic::ModuleDiagnosticV1;

/// A single declared parameter of a signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureParamV1 {
    /// The parameter's local name (used for diagnostics and labeled binding).
    pub name: String,
    /// An opaque symbolic type tag. Closed legality of this tag is ticket 03's
    /// authority; this crate only carries it through instantiation.
    pub type_tag: String,
}

impl SignatureParamV1 {
    /// Construct a parameter.
    #[must_use]
    pub fn new(name: impl Into<String>, type_tag: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_tag: type_tag.into(),
        }
    }
}

/// A parameterized signature / interface a reusable module abstraction exposes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureV1 {
    /// The signature name (unique within its module).
    pub name: String,
    /// Ordered formal parameters.
    pub params: Vec<SignatureParamV1>,
    /// The opaque symbolic result tag the abstraction produces.
    pub result_tag: String,
}

/// A structurally-checked instantiation of a signature.
///
/// Produced only by [`SignatureV1::instantiate`]; holds the signature name and
/// the per-parameter `(param, argument_tag)` bindings in declaration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureInstanceV1 {
    /// The instantiated signature name.
    pub signature: String,
    /// Bound `(parameter, argument tag)` pairs, in declaration order.
    pub bindings: Vec<(SignatureParamV1, String)>,
    /// The result tag carried from the signature.
    pub result_tag: String,
}

impl SignatureV1 {
    /// Construct a signature.
    #[must_use]
    pub fn new(name: impl Into<String>, result_tag: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            params: Vec::new(),
            result_tag: result_tag.into(),
        }
    }

    /// Append a formal parameter (builder style).
    #[must_use]
    pub fn with_param(mut self, param: SignatureParamV1) -> Self {
        self.params.push(param);
        self
    }

    /// Declared arity.
    #[must_use]
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    /// Instantiate this signature against positional argument tags.
    ///
    /// Fail-closed on arity mismatch with
    /// [`ModuleDiagnosticV1::SignatureArityMismatch`]. Type legality of the
    /// argument tags is NOT checked here (ticket 03 owns that); the binding is
    /// carried through for the typer to consume.
    pub fn instantiate(
        &self,
        argument_tags: &[String],
    ) -> Result<SignatureInstanceV1, ModuleDiagnosticV1> {
        if argument_tags.len() != self.params.len() {
            return Err(ModuleDiagnosticV1::SignatureArityMismatch {
                signature: self.name.clone(),
                expected: self.params.len(),
                found: argument_tags.len(),
            });
        }
        let bindings = self
            .params
            .iter()
            .cloned()
            .zip(argument_tags.iter().cloned())
            .collect();
        Ok(SignatureInstanceV1 {
            signature: self.name.clone(),
            bindings,
            result_tag: self.result_tag.clone(),
        })
    }
}
