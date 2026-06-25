//! The value-expression type checker (ticket 03, decisions `D4`/`D10`/`D29`).
//!
//! [`TypeCheckerV1`] assigns a closed [`TypeV1`] to a
//! [`quanta_gqlang_v2_hir::ValueExprHirV1`] against a caller-provided
//! [`TypeEnvV1`]. It owns the closed operator and builtin registries and runs
//! overload resolution; on failure it returns a typed [`TypeErrorV1`] with a
//! stable code and span (`D4`/`D10` — explicit reject, no runtime guessing).
//!
//! Per `D29` nested result-shape *structure* stays in ticket 01; here that
//! structure is *typed* through the shared record/list value nodes, so a record
//! `{ id: int, tags: list<string> }` types to the corresponding [`TypeV1`].
//!
//! Overload resolution is exact-match-first: a candidate overload matches when
//! every operand/argument type is assignable (per [`TypeV1::is_assignable_to`])
//! to the overload's declared type. The first matching overload in registry
//! order wins, keeping resolution total and order-deterministic.

use crate::builtin::{AggregationClassV1, BuiltinRegistryV1, BuiltinSignatureV1};
use crate::diag::{IllegalAggregateReasonV1, TypeErrorV1};
use crate::env::TypeEnvV1;
use crate::op::OperatorRegistryV1;
use crate::ty::{RecordFieldTyV1, ScalarTyV1, TypeV1};
use quanta_gqlang_v2_hir::{LiteralV1, PathAccessV1, SpanV1, ValueExprHirV1, ValueExprKindV1};

/// The closed type checker for value expressions.
#[derive(Clone, Debug)]
pub struct TypeCheckerV1 {
    operators: OperatorRegistryV1,
    builtins: BuiltinRegistryV1,
}

impl TypeCheckerV1 {
    /// Build a checker over the single closed operator and builtin registries.
    #[must_use]
    pub fn new() -> Self {
        Self {
            operators: OperatorRegistryV1::closed(),
            builtins: BuiltinRegistryV1::closed(),
        }
    }

    /// The closed operator registry this checker uses.
    #[must_use]
    pub fn operators(&self) -> &OperatorRegistryV1 {
        &self.operators
    }

    /// The closed builtin registry this checker uses.
    #[must_use]
    pub fn builtins(&self) -> &BuiltinRegistryV1 {
        &self.builtins
    }

    /// Type a value expression against `env`.
    pub fn type_value(
        &self,
        expr: &ValueExprHirV1,
        env: &TypeEnvV1,
    ) -> Result<TypeV1, TypeErrorV1> {
        match &expr.kind {
            ValueExprKindV1::Literal(lit) => self.type_literal(lit, expr.span),
            ValueExprKindV1::Symbol(symbol) => {
                env.lookup(&symbol.name)
                    .cloned()
                    .ok_or_else(|| TypeErrorV1::UnknownSymbol {
                        span: expr.span,
                        name: symbol.name.clone(),
                    })
            }
            ValueExprKindV1::Path(path) => self.type_path(path, env),
            ValueExprKindV1::Unary(unary) => {
                let operand = self.type_value(&unary.operand, env)?;
                self.resolve_unary(&unary.op.symbol, operand, expr.span)
            }
            ValueExprKindV1::Binary(binary) => {
                let lhs = self.type_value(&binary.lhs, env)?;
                let rhs = self.type_value(&binary.rhs, env)?;
                self.resolve_binary(&binary.op.symbol, lhs, rhs, expr.span)
            }
            ValueExprKindV1::Call(call) => {
                let mut args = Vec::with_capacity(call.args.len());
                for arg in &call.args {
                    args.push(self.type_value(arg, env)?);
                }
                self.resolve_call(&call.callee.name, &args, expr.span)
            }
            ValueExprKindV1::Paren(inner) => self.type_value(inner, env),
            ValueExprKindV1::Record(record) => {
                let mut fields = Vec::with_capacity(record.fields.len());
                for field in &record.fields {
                    let ty = self.type_value(&field.value, env)?;
                    fields.push(RecordFieldTyV1::new(field.name.clone(), ty));
                }
                Ok(TypeV1::record(fields))
            }
            ValueExprKindV1::List(list) => self.type_list(list, env, expr.span),
        }
    }

    fn type_literal(&self, lit: &LiteralV1, span: SpanV1) -> Result<TypeV1, TypeErrorV1> {
        match lit {
            LiteralV1::Int(text) => {
                if text.trim().parse::<i128>().is_ok() {
                    Ok(TypeV1::int())
                } else {
                    Err(TypeErrorV1::MalformedLiteral {
                        span,
                        expected: TypeV1::int(),
                        text: text.clone(),
                    })
                }
            }
            LiteralV1::Float(text) => {
                if text.trim().parse::<f64>().is_ok() {
                    Ok(TypeV1::float())
                } else {
                    Err(TypeErrorV1::MalformedLiteral {
                        span,
                        expected: TypeV1::float(),
                        text: text.clone(),
                    })
                }
            }
            LiteralV1::Str(_) => Ok(TypeV1::string()),
            LiteralV1::Bool(_) => Ok(TypeV1::bool()),
            LiteralV1::Null => Ok(TypeV1::Null),
        }
    }

    fn type_path(&self, path: &PathAccessV1, env: &TypeEnvV1) -> Result<TypeV1, TypeErrorV1> {
        let mut current = self.type_value(&path.base, env)?;
        for segment in &path.segments {
            current = self.project_field(&current, &segment.name, segment.span)?;
        }
        Ok(current)
    }

    /// Project `segment` out of a record-typed base, looking through one
    /// optional layer (`optional<{..}>` projects to `optional<field>`).
    fn project_field(
        &self,
        base: &TypeV1,
        segment: &str,
        span: SpanV1,
    ) -> Result<TypeV1, TypeErrorV1> {
        let (record_fields, wrap_optional) = match base {
            TypeV1::Record(fields) => (fields, false),
            TypeV1::Optional(inner) => {
                if let TypeV1::Record(fields) = inner.as_ref() {
                    (fields, true)
                } else {
                    return Err(self.bad_projection(base, segment, span));
                }
            }
            TypeV1::Scalar(_)
            | TypeV1::Null
            | TypeV1::Tuple(_)
            | TypeV1::Collection { .. }
            | TypeV1::Union(_)
            | TypeV1::Relation { .. } => {
                return Err(self.bad_projection(base, segment, span));
            }
        };
        match record_fields.iter().find(|field| field.name == segment) {
            Some(field) => {
                if wrap_optional {
                    Ok(TypeV1::optional(field.ty.clone()))
                } else {
                    Ok(field.ty.clone())
                }
            }
            None => Err(self.bad_projection(base, segment, span)),
        }
    }

    fn bad_projection(&self, base: &TypeV1, segment: &str, span: SpanV1) -> TypeErrorV1 {
        TypeErrorV1::BadProjection {
            span,
            base: base.clone(),
            segment: segment.to_owned(),
        }
    }

    fn type_list(
        &self,
        list: &quanta_gqlang_v2_hir::ListExprV1,
        env: &TypeEnvV1,
        span: SpanV1,
    ) -> Result<TypeV1, TypeErrorV1> {
        let mut element_ty: Option<TypeV1> = None;
        for element in &list.elements {
            let ty = self.type_value(element, env)?;
            match &element_ty {
                None => element_ty = Some(ty),
                Some(expected) => {
                    if expected.is_assignable_to(&ty) && !ty.is_assignable_to(expected) {
                        // `expected` widens into `ty` but not vice versa, so `ty`
                        // is the more general member; generalize the element type
                        // to keep the inferred `list<T>` assignable for every
                        // element seen so far.
                        element_ty = Some(ty);
                    } else if !ty.is_assignable_to(expected) {
                        return Err(TypeErrorV1::Mismatch {
                            span,
                            expected: expected.clone(),
                            actual: ty,
                        });
                    }
                }
            }
        }
        // An empty list has an unknown element type; model it as a list of the
        // null/absent type so it stays assignable into any concrete list via
        // later context. Non-empty lists carry their element type.
        let element = element_ty.unwrap_or(TypeV1::Null);
        Ok(TypeV1::list(element))
    }

    fn resolve_unary(
        &self,
        symbol: &str,
        operand: TypeV1,
        span: SpanV1,
    ) -> Result<TypeV1, TypeErrorV1> {
        if !self.operators.has_unary(symbol) {
            return Err(TypeErrorV1::UnknownUnaryOperator {
                span,
                operator: symbol.to_owned(),
            });
        }
        for overload in self.operators.unary_overloads(symbol) {
            if operand.is_assignable_to(&overload.operand) {
                return Ok(overload.result.clone());
            }
        }
        Err(TypeErrorV1::BadOverload {
            span,
            operator: symbol.to_owned(),
            operands: vec![operand],
        })
    }

    fn resolve_binary(
        &self,
        symbol: &str,
        lhs: TypeV1,
        rhs: TypeV1,
        span: SpanV1,
    ) -> Result<TypeV1, TypeErrorV1> {
        if !self.operators.has_binary(symbol) {
            return Err(TypeErrorV1::UnknownBinaryOperator {
                span,
                operator: symbol.to_owned(),
            });
        }
        for overload in self.operators.binary_overloads(symbol) {
            if lhs.is_assignable_to(&overload.lhs) && rhs.is_assignable_to(&overload.rhs) {
                return Ok(overload.result.clone());
            }
        }
        Err(TypeErrorV1::BadOverload {
            span,
            operator: symbol.to_owned(),
            operands: vec![lhs, rhs],
        })
    }

    fn resolve_call(
        &self,
        name: &str,
        args: &[TypeV1],
        span: SpanV1,
    ) -> Result<TypeV1, TypeErrorV1> {
        let overloads = self.builtins.lookup_all(name);
        if overloads.is_empty() {
            return Err(TypeErrorV1::UnknownBuiltin {
                span,
                name: name.to_owned(),
            });
        }
        // Determine whether ANY overload could accept this arity. If none can,
        // and the (first) entry is an aggregate, report illegal-aggregate arity;
        // otherwise a generic arity mismatch.
        let mut arity_ok = false;
        for entry in &overloads {
            if signature_accepts_arity(&entry.signature, args.len()) {
                arity_ok = true;
                if let Some(result) = match_signature(&entry.signature, args) {
                    return Ok(result);
                }
            }
        }
        // No overload matched. Distinguish the reject class.
        let first = overloads
            .first()
            .ok_or_else(|| TypeErrorV1::UnknownBuiltin {
                span,
                name: name.to_owned(),
            })?;
        if first.meta.aggregation == AggregationClassV1::Aggregate {
            if !arity_ok {
                let expected = first.signature.min_arity();
                return Err(TypeErrorV1::IllegalAggregate {
                    span,
                    function: name.to_owned(),
                    reason: IllegalAggregateReasonV1::BadArity {
                        expected,
                        actual: args.len(),
                    },
                });
            }
            let argument = args.first().cloned().unwrap_or(TypeV1::Null);
            return Err(TypeErrorV1::IllegalAggregate {
                span,
                function: name.to_owned(),
                reason: IllegalAggregateReasonV1::NonAggregableArgument { argument },
            });
        }
        if !arity_ok {
            let expected = first.signature.min_arity();
            return Err(TypeErrorV1::ArityMismatch {
                span,
                function: name.to_owned(),
                expected,
                actual: args.len(),
            });
        }
        Err(TypeErrorV1::BadOverload {
            span,
            operator: name.to_owned(),
            operands: args.to_vec(),
        })
    }
}

impl Default for TypeCheckerV1 {
    fn default() -> Self {
        Self::new()
    }
}

fn signature_accepts_arity(signature: &BuiltinSignatureV1, arity: usize) -> bool {
    if arity < signature.min_arity() {
        return false;
    }
    match signature.max_arity() {
        Some(max) => arity <= max,
        None => true,
    }
}

/// Try to match a fully-typed argument list against one signature, returning the
/// result type when every argument is assignable. Returns `None` on any
/// mismatch (arity or type).
fn match_signature(signature: &BuiltinSignatureV1, args: &[TypeV1]) -> Option<TypeV1> {
    if !signature_accepts_arity(signature, args.len()) {
        return None;
    }
    if signature.accepts_any_single {
        // Exactly one argument of any type; arity already checked.
        return Some(signature.result.clone());
    }
    let fixed = signature.params.len();
    for (idx, param) in signature.params.iter().enumerate() {
        let arg = args.get(idx)?;
        if !arg.is_assignable_to(param) {
            return None;
        }
    }
    if let Some(variadic) = &signature.variadic {
        for arg in args.iter().skip(fixed) {
            if !arg.is_assignable_to(variadic) {
                return None;
            }
        }
    }
    Some(signature.result.clone())
}

/// A scalar convenience used by callers building simple environments and by
/// path/proof fixtures: the closed scalar set is re-exported here so downstream
/// tickets do not need to reach into [`crate::ty`] for the common case.
#[must_use]
pub fn scalar(scalar: ScalarTyV1) -> TypeV1 {
    TypeV1::scalar(scalar)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string() -> TypeV1 {
        TypeV1::scalar(ScalarTyV1::String)
    }

    fn boolean() -> TypeV1 {
        TypeV1::scalar(ScalarTyV1::Bool)
    }

    #[test]
    fn match_signature_accepts_correct_arity_and_rejects_wrong_arity() {
        // Pins check.rs:372 `if !signature_accepts_arity(..)`. The `!` must be
        // present: with correct arity + assignable args the signature MATCHES
        // (Some), and with wrong arity it REJECTS (None). Deleting `!` flips the
        // first case to None, which this asserts against.
        let sig = BuiltinSignatureV1::fixed(vec![string(), string()], boolean());

        // Correct arity (2) with assignable args => Some(result).
        let ok = match_signature(&sig, &[string(), string()]);
        assert_eq!(
            ok,
            Some(boolean()),
            "a 2-string signature must match exactly two string args"
        );

        // Wrong arity (1, below min 2) => None.
        let too_few = match_signature(&sig, &[string()]);
        assert_eq!(too_few, None, "one arg must not match a 2-arg signature");

        // Wrong arity (3, above max 2) => None.
        let too_many = match_signature(&sig, &[string(), string(), string()]);
        assert_eq!(
            too_many, None,
            "three args must not match a 2-arg signature"
        );
    }

    #[test]
    fn match_signature_checks_variadic_argument_types() {
        // Pins check.rs:372 `if !arg.is_assignable_to(variadic)` INSIDE the
        // variadic loop (distinct from the arity `!` at :356). The closed
        // registry has no variadic builtin, so this drives the variadic path
        // directly. Signature: one fixed `string`, then any number of `int`s.
        let int = TypeV1::scalar(ScalarTyV1::Int);
        let sig = BuiltinSignatureV1 {
            params: vec![string()],
            variadic: Some(int.clone()),
            accepts_any_single: false,
            result: boolean(),
        };

        // Fixed string + two assignable int variadic args => MATCHES. Deleting
        // the `!` flips an assignable variadic arg to an early `None`, so this
        // Some assertion kills that mutant.
        let ok = match_signature(&sig, &[string(), int.clone(), int.clone()]);
        assert_eq!(
            ok,
            Some(boolean()),
            "int variadic args after the fixed string must be accepted"
        );

        // A variadic arg of the WRONG type (string where int is required) must
        // REJECT. Without the `!` this wrongly accepts, so this None assertion
        // also pins the variadic type gate.
        let bad = match_signature(&sig, &[string(), string()]);
        assert_eq!(
            bad, None,
            "a non-int variadic arg must be rejected by the variadic type gate"
        );

        // Zero variadic args (just the fixed param) still matches.
        let none_variadic = match_signature(&sig, &[string()]);
        assert_eq!(
            none_variadic,
            Some(boolean()),
            "the fixed param alone, with no variadic args, must match"
        );
    }

    #[test]
    fn signature_accepts_arity_boundaries() {
        // Backstops the arity gate used by match_signature: fixed-2 admits
        // exactly 2, any_single admits exactly 1.
        let fixed2 = BuiltinSignatureV1::fixed(vec![string(), string()], boolean());
        assert!(!signature_accepts_arity(&fixed2, 1));
        assert!(signature_accepts_arity(&fixed2, 2));
        assert!(!signature_accepts_arity(&fixed2, 3));

        let any1 = BuiltinSignatureV1::any_single(TypeV1::scalar(ScalarTyV1::Int));
        assert!(!signature_accepts_arity(&any1, 0));
        assert!(signature_accepts_arity(&any1, 1));
        assert!(!signature_accepts_arity(&any1, 2));
    }
}
