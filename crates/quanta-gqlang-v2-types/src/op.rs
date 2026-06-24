//! The closed operator registry (ticket 03, decision `D4`).
//!
//! Operators in the HIR are open symbolic tags (ticket 01); this module is the
//! single closed authority that decides which operator symbols exist and which
//! operand-type combinations they admit. Lookup is pure and backend-free
//! (`D4`): given an operator symbol and operand types, the registry either
//! returns the result type of the unique matching overload or reports that no
//! overload matched (`bad overload`) / the operator is unknown.
//!
//! The registry is data, built once. Each overload is an exact-match signature
//! over the closed [`TypeV1`] universe; overload resolution (in [`crate::check`])
//! is exact-match-first with a small set of explicit, total promotion rules so
//! behavior is predictable for downstream planning.

use crate::ty::{ScalarTyV1, TypeV1};

/// The kind of unary operator overload result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryOverloadV1 {
    /// Operand type this overload accepts.
    pub operand: TypeV1,
    /// Result type produced.
    pub result: TypeV1,
}

/// A binary operator overload signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryOverloadV1 {
    /// Left operand type.
    pub lhs: TypeV1,
    /// Right operand type.
    pub rhs: TypeV1,
    /// Result type produced.
    pub result: TypeV1,
}

/// The closed operator registry.
#[derive(Clone, Debug)]
pub struct OperatorRegistryV1 {
    unary: Vec<(String, UnaryOverloadV1)>,
    binary: Vec<(String, BinaryOverloadV1)>,
}

impl OperatorRegistryV1 {
    /// Build the single closed operator registry.
    #[must_use]
    pub fn closed() -> Self {
        let mut registry = Self {
            unary: Vec::new(),
            binary: Vec::new(),
        };
        registry.install();
        registry
    }

    /// All overloads for a unary operator symbol (possibly empty).
    #[must_use]
    pub fn unary_overloads(&self, symbol: &str) -> Vec<&UnaryOverloadV1> {
        self.unary
            .iter()
            .filter(|(sym, _)| sym == symbol)
            .map(|(_, ov)| ov)
            .collect()
    }

    /// All overloads for a binary operator symbol (possibly empty).
    #[must_use]
    pub fn binary_overloads(&self, symbol: &str) -> Vec<&BinaryOverloadV1> {
        self.binary
            .iter()
            .filter(|(sym, _)| sym == symbol)
            .map(|(_, ov)| ov)
            .collect()
    }

    /// Whether the registry knows this unary operator symbol at all.
    #[must_use]
    pub fn has_unary(&self, symbol: &str) -> bool {
        self.unary.iter().any(|(sym, _)| sym == symbol)
    }

    /// Whether the registry knows this binary operator symbol at all.
    #[must_use]
    pub fn has_binary(&self, symbol: &str) -> bool {
        self.binary.iter().any(|(sym, _)| sym == symbol)
    }

    fn add_unary(&mut self, symbol: &str, operand: TypeV1, result: TypeV1) {
        self.unary
            .push((symbol.to_owned(), UnaryOverloadV1 { operand, result }));
    }

    fn add_binary(&mut self, symbol: &str, lhs: TypeV1, rhs: TypeV1, result: TypeV1) {
        self.binary
            .push((symbol.to_owned(), BinaryOverloadV1 { lhs, rhs, result }));
    }

    fn install(&mut self) {
        let int = || TypeV1::scalar(ScalarTyV1::Int);
        let float = || TypeV1::scalar(ScalarTyV1::Float);
        let boolean = || TypeV1::scalar(ScalarTyV1::Bool);
        let string = || TypeV1::scalar(ScalarTyV1::String);
        let duration = || TypeV1::scalar(ScalarTyV1::Duration);

        // ---- arithmetic unary ------------------------------------------------
        for sym in ["-", "+"] {
            self.add_unary(sym, int(), int());
            self.add_unary(sym, float(), float());
            self.add_unary(sym, duration(), duration());
        }

        // ---- boolean unary ---------------------------------------------------
        self.add_unary("not", boolean(), boolean());

        // ---- arithmetic binary ----------------------------------------------
        for sym in ["+", "-", "*"] {
            self.add_binary(sym, int(), int(), int());
            self.add_binary(sym, float(), float(), float());
            self.add_binary(sym, duration(), duration(), duration());
        }
        // division promotes integer division result type to int, float to float.
        self.add_binary("/", int(), int(), int());
        self.add_binary("/", float(), float(), float());
        // duration scaling by a number.
        self.add_binary("*", duration(), int(), duration());
        self.add_binary("/", duration(), int(), duration());
        // modulo on integers only.
        self.add_binary("%", int(), int(), int());

        // ---- comparison (ordering) ------------------------------------------
        for sym in ["<", "<=", ">", ">="] {
            self.add_binary(sym, int(), int(), boolean());
            self.add_binary(sym, float(), float(), boolean());
            self.add_binary(sym, string(), string(), boolean());
            self.add_binary(sym, duration(), duration(), boolean());
        }
        // ---- equality (closed scalar set) -----------------------------------
        for sym in ["=", "!="] {
            for scalar in [
                ScalarTyV1::Bool,
                ScalarTyV1::Int,
                ScalarTyV1::Float,
                ScalarTyV1::String,
                ScalarTyV1::Duration,
                ScalarTyV1::Location,
                ScalarTyV1::Path,
                ScalarTyV1::EntityRef,
                ScalarTyV1::SymbolRef,
                ScalarTyV1::RuleRef,
                ScalarTyV1::SourceScope,
                ScalarTyV1::ProofTree,
            ] {
                self.add_binary(
                    sym,
                    TypeV1::scalar(scalar.clone()),
                    TypeV1::scalar(scalar),
                    boolean(),
                );
            }
        }

        // ---- boolean binary -------------------------------------------------
        for sym in ["and", "or", "xor"] {
            self.add_binary(sym, boolean(), boolean(), boolean());
        }

        // ---- string concatenation -------------------------------------------
        self.add_binary("++", string(), string(), string());
    }
}

impl Default for OperatorRegistryV1 {
    fn default() -> Self {
        Self::closed()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests assert on Option directly")]
    use super::*;

    #[test]
    fn arithmetic_plus_has_int_overload() {
        let registry = OperatorRegistryV1::closed();
        assert!(registry.has_binary("+"));
        let overloads = registry.binary_overloads("+");
        assert!(overloads
            .iter()
            .any(|ov| ov.lhs == TypeV1::int() && ov.result == TypeV1::int()));
    }

    #[test]
    fn unknown_operator_has_no_overloads() {
        let registry = OperatorRegistryV1::closed();
        assert!(!registry.has_binary("<=>"));
        assert!(registry.binary_overloads("<=>").is_empty());
    }

    #[test]
    fn ordering_is_bool_typed() {
        let registry = OperatorRegistryV1::closed();
        let lt = registry.binary_overloads("<");
        assert!(lt.iter().all(|ov| ov.result == TypeV1::bool()));
    }
}
