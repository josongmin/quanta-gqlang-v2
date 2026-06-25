//! The closed builtin function registry with metadata (ticket 03).
//!
//! Per the ticket, every builtin carries the metadata that decisions `D21`,
//! `D24`, and downstream tickets `05`/`07` consume: purity, determinism,
//! monotonicity, aggregation class, and cost class. The registry is the single
//! closed authority for which builtins exist and what types they accept/produce;
//! lookup is pure and backend-free (`D4`).
//!
//! Signature shape: each builtin has a fixed list of parameter types and a
//! result type. Variadic builtins (e.g. `count(*)`) are modelled with an
//! explicit [`BuiltinSignatureV1::variadic`] flag; `count` accepts any single
//! argument and is the only nullary-capable aggregate via the dedicated
//! [`BuiltinSignatureV1::min_arity`].

use crate::ty::{CardinalityV1, ScalarTyV1, TypeV1};

/// Whether a builtin reads only its arguments (no ambient/world state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PurityV1 {
    /// Pure: result depends only on argument values.
    Pure,
    /// Impure: result may depend on ambient state (e.g. clock, source scan).
    Impure,
}

/// Whether a builtin always yields the same output for the same input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeterminismV1 {
    /// Deterministic: identical inputs always give identical outputs.
    Deterministic,
    /// Non-deterministic: outputs may vary across evaluations.
    NonDeterministic,
}

/// Monotonicity class, consumed by recursion legality (`05`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonotonicityV1 {
    /// Output grows (never shrinks) as inputs grow — safe in least-fixpoint.
    Monotone,
    /// Output may shrink as inputs grow (e.g. negation, `not`).
    Antitone,
    /// No monotonicity guarantee.
    NonMonotone,
}

/// Aggregation class of a builtin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregationClassV1 {
    /// Not an aggregate: maps argument values to a scalar result row-wise.
    Scalar,
    /// An aggregate that folds a collection of rows into one value.
    Aggregate,
}

/// Cost class, consumed by cost diagnostics (`07`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostClassV1 {
    /// Constant / negligible cost.
    Constant,
    /// Linear in input size.
    Linear,
    /// Super-linear (e.g. sort/path search).
    SuperLinear,
}

/// Metadata block attached to every builtin entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinMetaV1 {
    /// Purity class.
    pub purity: PurityV1,
    /// Determinism class.
    pub determinism: DeterminismV1,
    /// Monotonicity class.
    pub monotonicity: MonotonicityV1,
    /// Aggregation class.
    pub aggregation: AggregationClassV1,
    /// Cost class.
    pub cost: CostClassV1,
}

impl BuiltinMetaV1 {
    /// The common case: a pure, deterministic, monotone, scalar, constant-cost
    /// builtin.
    #[must_use]
    pub const fn pure_scalar(cost: CostClassV1) -> Self {
        Self {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity: MonotonicityV1::Monotone,
            aggregation: AggregationClassV1::Scalar,
            cost,
        }
    }

    /// Whether this entry is an aggregate.
    #[must_use]
    pub const fn is_aggregate(&self) -> bool {
        matches!(self.aggregation, AggregationClassV1::Aggregate)
    }

    /// Whether this entry is pure and deterministic (safe-by-default).
    #[must_use]
    pub const fn is_pure_deterministic(&self) -> bool {
        matches!(self.purity, PurityV1::Pure)
            && matches!(self.determinism, DeterminismV1::Deterministic)
    }
}

/// A builtin's typed signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinSignatureV1 {
    /// Fixed leading parameter types.
    pub params: Vec<TypeV1>,
    /// When `Some`, additional trailing arguments of this type are accepted.
    pub variadic: Option<TypeV1>,
    /// When `true`, a single argument of ANY type is accepted (used by
    /// `count`); `params` must then be empty.
    pub accepts_any_single: bool,
    /// Result type.
    pub result: TypeV1,
}

impl BuiltinSignatureV1 {
    /// A fixed-arity signature.
    #[must_use]
    pub fn fixed(params: Vec<TypeV1>, result: TypeV1) -> Self {
        Self {
            params,
            variadic: None,
            accepts_any_single: false,
            result,
        }
    }

    /// A signature that accepts exactly one argument of any type.
    #[must_use]
    pub fn any_single(result: TypeV1) -> Self {
        Self {
            params: Vec::new(),
            variadic: None,
            accepts_any_single: true,
            result,
        }
    }

    /// Minimum number of arguments accepted.
    #[must_use]
    pub fn min_arity(&self) -> usize {
        if self.accepts_any_single {
            1
        } else {
            self.params.len()
        }
    }

    /// Maximum number of arguments accepted; `None` when variadic-unbounded.
    #[must_use]
    pub fn max_arity(&self) -> Option<usize> {
        if self.variadic.is_some() {
            None
        } else if self.accepts_any_single {
            Some(1)
        } else {
            Some(self.params.len())
        }
    }
}

/// A single builtin registry entry: name, signature, and metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinEntryV1 {
    /// Builtin name as written at the callee.
    pub name: String,
    /// Typed signature.
    pub signature: BuiltinSignatureV1,
    /// Metadata block.
    pub meta: BuiltinMetaV1,
}

/// The closed builtin function registry.
#[derive(Clone, Debug)]
pub struct BuiltinRegistryV1 {
    entries: Vec<BuiltinEntryV1>,
}

impl BuiltinRegistryV1 {
    /// Build the single closed builtin registry.
    #[must_use]
    pub fn closed() -> Self {
        let mut registry = Self {
            entries: Vec::new(),
        };
        registry.install();
        registry
    }

    /// Look up the first builtin entry by exact name.
    ///
    /// Some names (e.g. numeric aggregates) carry several overloads; use
    /// [`BuiltinRegistryV1::lookup_all`] to inspect the full overload set.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&BuiltinEntryV1> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    /// All builtin entries (overloads) sharing this exact name, in install
    /// order.
    #[must_use]
    pub fn lookup_all(&self, name: &str) -> Vec<&BuiltinEntryV1> {
        self.entries
            .iter()
            .filter(|entry| entry.name == name)
            .collect()
    }

    /// Whether the registry knows this builtin name.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// All entries, in install order (for documentation/introspection).
    #[must_use]
    pub fn entries(&self) -> &[BuiltinEntryV1] {
        &self.entries
    }

    fn add(&mut self, name: &str, signature: BuiltinSignatureV1, meta: BuiltinMetaV1) {
        self.entries.push(BuiltinEntryV1 {
            name: name.to_owned(),
            signature,
            meta,
        });
    }

    fn install(&mut self) {
        let int = || TypeV1::scalar(ScalarTyV1::Int);
        let float = || TypeV1::scalar(ScalarTyV1::Float);
        let boolean = || TypeV1::scalar(ScalarTyV1::Bool);
        let string = || TypeV1::scalar(ScalarTyV1::String);
        let location = || TypeV1::scalar(ScalarTyV1::Location);
        let path = || TypeV1::scalar(ScalarTyV1::Path);
        let proof = || TypeV1::scalar(ScalarTyV1::ProofTree);

        // ---- aggregate family -----------------------------------------------
        let aggregate_meta = BuiltinMetaV1 {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity: MonotonicityV1::Monotone,
            aggregation: AggregationClassV1::Aggregate,
            cost: CostClassV1::Linear,
        };
        // count(any) -> int   (count(*) is modelled as count of one arg).
        self.add(
            "count",
            BuiltinSignatureV1::any_single(int()),
            aggregate_meta,
        );
        // sum/min/max over int and float — registered as an overload set per
        // name; the checker picks the overload whose param type the argument is
        // assignable to (int->int, float->float).
        for sym in ["sum", "min", "max"] {
            self.add(
                sym,
                BuiltinSignatureV1::fixed(vec![int()], int()),
                aggregate_meta,
            );
            self.add(
                sym,
                BuiltinSignatureV1::fixed(vec![float()], float()),
                aggregate_meta,
            );
        }
        // avg always yields float, from int or float input.
        self.add(
            "avg",
            BuiltinSignatureV1::fixed(vec![int()], float()),
            aggregate_meta,
        );
        self.add(
            "avg",
            BuiltinSignatureV1::fixed(vec![float()], float()),
            aggregate_meta,
        );

        // ---- string predicates ----------------------------------------------
        self.add(
            "starts_with",
            BuiltinSignatureV1::fixed(vec![string(), string()], boolean()),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );
        self.add(
            "ends_with",
            BuiltinSignatureV1::fixed(vec![string(), string()], boolean()),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );
        self.add(
            "contains",
            BuiltinSignatureV1::fixed(vec![string(), string()], boolean()),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );
        self.add(
            "length",
            BuiltinSignatureV1::fixed(vec![string()], int()),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );

        // ---- null / default helpers -----------------------------------------
        // coalesce(optional<T>, T) -> T is type-directed; checker specializes T.
        self.add(
            "is_null",
            BuiltinSignatureV1::any_single(boolean()),
            BuiltinMetaV1::pure_scalar(CostClassV1::Constant),
        );

        // ---- collection predicates ------------------------------------------
        self.add(
            "size",
            BuiltinSignatureV1::fixed(
                vec![TypeV1::collection(int(), CardinalityV1::Sequence)],
                int(),
            ),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );

        // ---- path / witness helpers -----------------------------------------
        self.add(
            "path_length",
            BuiltinSignatureV1::fixed(vec![path()], int()),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );
        self.add(
            "path_locations",
            BuiltinSignatureV1::fixed(
                vec![path()],
                TypeV1::collection(location(), CardinalityV1::Sequence),
            ),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );

        // ---- proof / explain helpers ----------------------------------------
        self.add(
            "explain",
            BuiltinSignatureV1::fixed(vec![boolean()], proof()),
            BuiltinMetaV1 {
                purity: PurityV1::Pure,
                determinism: DeterminismV1::Deterministic,
                monotonicity: MonotonicityV1::NonMonotone,
                aggregation: AggregationClassV1::Scalar,
                cost: CostClassV1::SuperLinear,
            },
        );
        self.add(
            "proof_size",
            BuiltinSignatureV1::fixed(vec![proof()], int()),
            BuiltinMetaV1::pure_scalar(CostClassV1::Linear),
        );

        // ---- non-deterministic example (visible to planning) ----------------
        self.add(
            "now",
            BuiltinSignatureV1::fixed(vec![], TypeV1::scalar(ScalarTyV1::Duration)),
            BuiltinMetaV1 {
                purity: PurityV1::Impure,
                determinism: DeterminismV1::NonDeterministic,
                monotonicity: MonotonicityV1::NonMonotone,
                aggregation: AggregationClassV1::Scalar,
                cost: CostClassV1::Constant,
            },
        );
    }
}

impl Default for BuiltinRegistryV1 {
    fn default() -> Self {
        Self::closed()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests assert on Option directly")]
    use super::*;

    #[test]
    fn count_is_aggregate_over_any_single_arg() {
        let registry = BuiltinRegistryV1::closed();
        let count = registry.lookup("count").expect("count exists");
        assert!(count.meta.is_aggregate());
        assert!(count.signature.accepts_any_single);
        assert_eq!(count.signature.min_arity(), 1);
        assert_eq!(count.signature.max_arity(), Some(1));
    }

    #[test]
    fn numeric_aggregates_have_int_and_float_overloads() {
        let registry = BuiltinRegistryV1::closed();
        let sums = registry.lookup_all("sum");
        assert_eq!(sums.len(), 2);
        assert!(sums.iter().all(|e| e.meta.is_aggregate()));
    }

    #[test]
    fn now_is_impure_nondeterministic() {
        let registry = BuiltinRegistryV1::closed();
        let now = registry.lookup("now").expect("now exists");
        assert!(!now.meta.is_pure_deterministic());
        assert_eq!(now.meta.purity, PurityV1::Impure);
        assert_eq!(now.meta.determinism, DeterminismV1::NonDeterministic);
    }

    #[test]
    fn unknown_builtin_absent() {
        let registry = BuiltinRegistryV1::closed();
        assert!(!registry.has("frobnicate"));
        assert!(registry.lookup_all("frobnicate").is_empty());
    }

    #[test]
    fn scalar_builtin_is_not_aggregate() {
        // Pins builtin.rs:98 `is_aggregate`: a Scalar builtin must report false.
        // (If the body were hardcoded to `true`, this would fail.)
        let registry = BuiltinRegistryV1::closed();
        let starts_with = registry.lookup("starts_with").expect("starts_with exists");
        assert_eq!(starts_with.meta.aggregation, AggregationClassV1::Scalar);
        assert!(!starts_with.meta.is_aggregate());
    }

    #[test]
    fn is_pure_deterministic_requires_both_pure_and_deterministic() {
        // Pins builtin.rs:105 `&&` vs `||`. We exercise all four corners so the
        // boundary between AND and OR is observable:
        //   pure   + deterministic    => true  (AND and OR agree)
        //   impure + deterministic    => AND=false, OR=true   (distinguishes)
        //   pure   + nondeterministic => AND=false, OR=true   (distinguishes)
        //   impure + nondeterministic => false (AND and OR agree)
        let mk = |purity, determinism| BuiltinMetaV1 {
            purity,
            determinism,
            monotonicity: MonotonicityV1::Monotone,
            aggregation: AggregationClassV1::Scalar,
            cost: CostClassV1::Constant,
        };
        assert!(mk(PurityV1::Pure, DeterminismV1::Deterministic).is_pure_deterministic());
        assert!(
            !mk(PurityV1::Impure, DeterminismV1::Deterministic).is_pure_deterministic(),
            "impure-but-deterministic must be false (AND), not true (OR)"
        );
        assert!(
            !mk(PurityV1::Pure, DeterminismV1::NonDeterministic).is_pure_deterministic(),
            "pure-but-nondeterministic must be false (AND), not true (OR)"
        );
        assert!(!mk(PurityV1::Impure, DeterminismV1::NonDeterministic).is_pure_deterministic());
    }

    #[test]
    fn max_arity_reflects_signature_shape() {
        // Pins builtin.rs:159 `max_arity`: distinct values for the three shapes,
        // so it cannot collapse to a constant None / Some(0) / Some(1).
        // Fixed binary signature => Some(2).
        let fixed2 = BuiltinSignatureV1::fixed(
            vec![
                TypeV1::scalar(ScalarTyV1::String),
                TypeV1::scalar(ScalarTyV1::String),
            ],
            TypeV1::scalar(ScalarTyV1::Bool),
        );
        assert_eq!(fixed2.max_arity(), Some(2));
        // accepts_any_single => Some(1).
        let any1 = BuiltinSignatureV1::any_single(TypeV1::scalar(ScalarTyV1::Int));
        assert_eq!(any1.max_arity(), Some(1));
        // variadic-unbounded => None.
        let variadic = BuiltinSignatureV1 {
            params: vec![TypeV1::scalar(ScalarTyV1::Int)],
            variadic: Some(TypeV1::scalar(ScalarTyV1::Int)),
            accepts_any_single: false,
            result: TypeV1::scalar(ScalarTyV1::Int),
        };
        assert_eq!(variadic.max_arity(), None);
    }

    #[test]
    fn entries_exposes_real_registry_contents() {
        // Pins builtin.rs:225 `entries`: must return the actual installed
        // entries, not an empty slice or a default-filled one.
        let registry = BuiltinRegistryV1::closed();
        let entries = registry.entries();
        assert!(
            entries.len() > 10,
            "registry must expose its many real entries, got {}",
            entries.len()
        );
        // A specific real entry must be present with its real name.
        assert!(
            entries.iter().any(|e| e.name == "count"),
            "entries() must include the real `count` builtin"
        );
        assert!(entries.iter().any(|e| e.name == "now"));
        // Slice length matches the lookup count for a name with overloads.
        assert_eq!(
            entries.iter().filter(|e| e.name == "sum").count(),
            registry.lookup_all("sum").len()
        );
    }
}
