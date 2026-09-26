#![expect(
    clippy::expect_used,
    reason = "assert checked owner fixtures and worker results"
)]

use quanta_gqlang_v2_hir::{
    check_document_v2, CallArgsHirV1, HirBuilderV1, HirDocumentV1, ProgramItemHirV1, SourceIdV2,
    SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, BuiltinRegistryV2, CostClassV1, DeterminismV1, ExpressionEffectsV2,
    ExpressionTypeKernelV2, ExpressionTypeStateV2, MonotonicityV1, PurityV1,
    ResolvedCallableInputV2, ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2,
    ResolvedTypeInputsV2, ScalarTyV1, SourceQualifiedNodeIdV2, TypeDiagnosticV2, ValueTypeV1,
};

const SOURCE: SourceIdV2 = SourceIdV2::from_raw(7);
const SPAN: SpanV1 = SpanV1::new(0, 1);

/// Each body has syntax depth two. Dependency depth is independently 4096.
fn chain(cyclic: bool, reverse: bool) {
    const COUNT: usize = 4096;
    let builder = HirBuilderV1::new();
    let registry = BuiltinRegistryV2::try_closed().expect("builtin registry");
    let mut items = Vec::new();
    let mut roots = Vec::new();
    let mut references = Vec::new();
    let mut entries = Vec::new();
    let key = |node_id| SourceQualifiedNodeIdV2 {
        source_id: SOURCE,
        node_id,
    };
    for index in 0..COUNT {
        let body = if index + 1 == COUNT && !cyclic {
            let call = builder.call(
                "now",
                SPAN,
                CallArgsHirV1::try_new(vec![]).expect("no args"),
                SPAN,
            );
            entries.push(ResolvedTypeInputEntryV2 {
                key: key(call.id),
                value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
                    registry.family_id("now").expect("now family"),
                )),
            });
            call
        } else {
            let reference = builder.symbol(format!("x{}", (index + 1) % COUNT), SPAN);
            references.push(key(reference.id));
            builder.paren_value(reference, SPAN)
        };
        roots.push(key(body.id));
        items.push(ProgramItemHirV1::from(builder.decl(
            format!("x{index}"),
            SPAN,
            body,
            SPAN,
        )));
    }
    for (reference, target) in references.iter().zip(roots.iter().cycle().skip(1)) {
        entries.push(ResolvedTypeInputEntryV2 {
            key: *reference,
            value: ResolvedTypeInputValueV2::BindingFromDeclaration(*target),
        });
    }
    if reverse {
        items.reverse();
    }
    let document = HirDocumentV1 {
        source_id: SOURCE,
        source_len: 1,
        items,
    };
    let registry = SourceRegistryViewV2::try_new(vec![])
        .expect("host registry")
        .try_extend_document(SourceRegistrationV2 {
            source_id: SOURCE,
            source_len: 1,
        })
        .expect("document source");
    let inputs = ResolvedTypeInputsV2::try_new(
        check_document_v2(&document).expect("shallow valid HIR"),
        &registry,
        entries,
        vec![],
        vec![],
    )
    .expect("resolved dependencies");
    let result = ExpressionTypeKernelV2::try_closed()
        .expect("type kernel")
        .check_document(&inputs);
    if cyclic {
        let first = roots.first().expect("first body");
        assert_eq!(
            result.expect_err("cycle rejects"),
            TypeDiagnosticV2::CyclicDeclarationDependency {
                node_id: first.node_id,
                location: quanta_gqlang_v2_hir::LocatedSpanV2 {
                    source_id: SOURCE,
                    span: SPAN
                },
            }
        );
        return;
    }
    let facts = result.expect("acyclic declarations type");
    assert_eq!(facts.len(), COUNT * 2 - 1);
    let expected_effects = ExpressionEffectsV2 {
        purity: PurityV1::Impure,
        determinism: DeterminismV1::NonDeterministic,
        monotonicity: MonotonicityV1::NonMonotone,
        aggregation: AggregationClassV1::Scalar,
        cost: CostClassV1::Constant,
    };
    for key in roots.iter().chain(&references) {
        let fact = facts.get(*key).expect("every dependency has one fact");
        assert_eq!(
            fact.state,
            ExpressionTypeStateV2::Known(ValueTypeV1::scalar(ScalarTyV1::Duration))
        );
        assert_eq!(
            fact.effects, expected_effects,
            "effect must survive each alias"
        );
    }
}

#[test]
fn long_declaration_chains_preserve_types_and_effects_in_both_orders() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            chain(false, false);
            chain(false, true);
        })
        .expect("bounded stack worker")
        .join()
        .expect("worker completes without overflow");
}

#[test]
fn long_declaration_cycle_retains_exact_typed_rejection() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| chain(true, false))
        .expect("bounded stack worker")
        .join()
        .expect("cycle does not overflow");
}

#[test]
fn collection_error_precedes_a_later_cyclic_dependency() {
    let builder = HirBuilderV1::new();
    let reference = builder.symbol("cycle", SPAN);
    let cycle = builder.symbol("cycle", SPAN);
    let key = |node_id| SourceQualifiedNodeIdV2 {
        source_id: SOURCE,
        node_id,
    };
    let entries = [key(reference.id), key(cycle.id)]
        .into_iter()
        .map(|key| ResolvedTypeInputEntryV2 {
            key,
            value: ResolvedTypeInputValueV2::BindingFromDeclaration(SourceQualifiedNodeIdV2 {
                source_id: SOURCE,
                node_id: cycle.id,
            }),
        })
        .collect();
    let body = builder.list(
        vec![
            builder.int("1", SPAN),
            builder.bool_lit(true, SPAN),
            reference,
        ],
        SPAN,
    );
    let root = body.id;
    let document = HirDocumentV1 {
        source_id: SOURCE,
        source_len: 1,
        items: vec![
            ProgramItemHirV1::from(builder.decl("first", SPAN, body, SPAN)),
            ProgramItemHirV1::from(builder.decl("cycle", SPAN, cycle, SPAN)),
        ],
    };
    let registry = SourceRegistryViewV2::try_new(vec![])
        .expect("host registry")
        .try_extend_document(SourceRegistrationV2 {
            source_id: SOURCE,
            source_len: 1,
        })
        .expect("source");
    let inputs = ResolvedTypeInputsV2::try_new(
        check_document_v2(&document).expect("HIR"),
        &registry,
        entries,
        vec![],
        vec![],
    )
    .expect("inputs");
    let error = ExpressionTypeKernelV2::try_closed()
        .expect("kernel")
        .check_document(&inputs)
        .expect_err("bad list");
    assert_eq!(
        error,
        TypeDiagnosticV2::HeterogeneousCollection {
            node_id: root,
            expected: ValueTypeV1::int(),
            actual: ValueTypeV1::bool(),
            location: quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SOURCE,
                span: SPAN
            },
        }
    );
}
