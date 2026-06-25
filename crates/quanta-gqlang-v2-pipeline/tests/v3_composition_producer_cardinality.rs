//! v3 lock rail (D26): a composition producer's lifted OUTPUT relation must
//! carry the producer's ACTUAL normalized cardinality, not a hardcoded `Bag`.
//!
//! Regression target `PIPELINE_COMPOSITION_CARDINALITY_MISLIFT`:
//! `compile_composition` lifted every relational producer's output schema with
//! `CardinalityV1::Bag` stamped unconditionally, regardless of the producer's
//! real multiplicity. An ORDER-BY producer denotes a `Sequence` (order becomes
//! semantic), so binding it as a `Bag` silently drops the producer's explicit
//! cardinality contract and violates D26 (multiplicity / duplicate-elimination
//! / output-order semantics must be explicit, not a guessed default).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration rail: assertion panics and indexing are intentional"
)]

use quanta_gqlang_v2_pipeline::{compile_program_with_schema, CompiledItemV1};
use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, TypeEnvV1, TypeV1};

/// `findings(repo: string)` as the only frontend schema authority, as a Bag so
/// a lifted Sequence cannot be inherited from the base source.
fn findings_schema() -> TypeEnvV1 {
    TypeEnvV1::new().with(
        "findings",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                "repo",
                TypeV1::string(),
            )])),
            cardinality: CardinalityV1::Bag,
        },
    )
}

/// The cardinality of the relation a binding output lifts.
fn lifted_cardinality(relation: &TypeV1) -> CardinalityV1 {
    let TypeV1::Relation { cardinality, .. } = relation else {
        panic!("a binding output must be a Relation, got {relation:?}")
    };
    *cardinality
}

/// An ORDER-BY producer denotes a `Sequence` (its compiled algebra root is a
/// Project over a Sort; sort makes order semantic). Its lifted binding relation
/// must therefore carry `Sequence` cardinality, not the previously hardcoded
/// `Bag`.
#[test]
fn ordered_producer_lifts_sequence_cardinality() {
    let src = "let x = from findings f | order by f.repo | project f.repo as repo\n\
               from x y | project y.repo as repo with x";
    let compiled = compile_program_with_schema(src, &findings_schema())
        .expect("composition with an ordered producer compiles");

    let CompiledItemV1::Composition {
        bindings,
        binding_outputs,
        ..
    } = compiled.compiled()
    else {
        panic!("source is a LET..WITH composition");
    };

    // Sanity: the producer's compiled algebra is itself a Sequence (the source
    // of truth the lifted relation must mirror).
    let CompiledItemV1::RelationalQuery { algebra, .. } = &bindings[0] else {
        panic!("the producer is a relational query");
    };
    assert_eq!(
        algebra.cardinality,
        CardinalityV1::Sequence,
        "an order-by producer's algebra root is a Sequence"
    );

    let lifted = binding_outputs[0]
        .as_ref()
        .expect("the producer projects a named column and lifts a relation");
    assert_eq!(
        lifted_cardinality(lifted),
        CardinalityV1::Sequence,
        "the lifted binding relation must mirror the producer's actual Sequence \
         cardinality, not a hardcoded Bag (D26)"
    );
}

/// Soundness companion: a plain `Bag` producer still lifts a `Bag` relation, so
/// the fix threads the real cardinality rather than inverting the default.
#[test]
fn bag_producer_lifts_bag_cardinality() {
    let src = "let x = from findings f | project f.repo as repo\n\
               from x y | project y.repo as repo with x";
    let compiled =
        compile_program_with_schema(src, &findings_schema()).expect("composition compiles");

    let CompiledItemV1::Composition {
        binding_outputs, ..
    } = compiled.compiled()
    else {
        panic!("source is a LET..WITH composition");
    };

    let lifted = binding_outputs[0]
        .as_ref()
        .expect("the producer projects a named column and lifts a relation");
    assert_eq!(
        lifted_cardinality(lifted),
        CardinalityV1::Bag,
        "a plain projection producer is a Bag"
    );
}
