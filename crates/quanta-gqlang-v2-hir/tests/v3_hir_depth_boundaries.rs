#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "depth-boundary tests require exact rejected outcomes and a launched child process"
)]

use quanta_gqlang_v2_hir::{
    check_tree, CheckedHirDocumentOwnedV2, HirBuilderV1, HirDepthDimensionV2, HirDocumentV1,
    HirIntegrityErrorV1, HirRootRefV1, ProgramItemHirV1, RelationExistsPolarityV1, SourceIdV2,
    SpanV1,
};

fn assert_exact_child_passes(test_name: &str, env_name: &str, mode: &str) {
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", test_name, "--nocapture"])
        .env(env_name, mode)
        .output()
        .expect("child process");
    assert!(
        output.status.success(),
        "{test_name} {mode} child status: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;"),
        "{test_name} {mode} selected zero or multiple tests: {stdout}"
    );
}

#[test]
fn raw_deep_drop_child() {
    let Some(mode) = std::env::var_os("QGLANG_RAW_HIR_DROP_CHILD") else {
        return;
    };
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    match mode.to_str().expect("known child mode") {
        "relation" | "document" => {
            let mut relation = builder.relation_source("r", span, None, span);
            for _ in 0..100_000 {
                relation =
                    builder.relation_filter(relation, builder.bool_literal(true, span), span);
            }
            if mode == "document" {
                let document = HirDocumentV1 {
                    source_id: SourceIdV2::from_raw(1),
                    source_len: 1,
                    items: vec![ProgramItemHirV1::from(builder.query(relation, None, span))],
                };
                drop(document);
            } else {
                drop(relation);
            }
        }
        "value" => {
            let mut value = builder.int("1", span);
            for _ in 0..100_000 {
                value = builder.paren_value(value, span);
            }
            drop(value);
        }
        "bool" => {
            let mut boolean = builder.bool_literal(true, span);
            for _ in 0..100_000 {
                boolean = builder.not(boolean, span);
            }
            drop(boolean);
        }
        "shape" => {
            let mut shape = builder.shape_leaf(builder.int("1", span), span);
            for _ in 0..100_000 {
                shape = builder.shape_list(shape, span);
            }
            drop(shape);
        }
        "mixed" => {
            let mut relation = builder.relation_source("r", span, None, span);
            for _ in 0..20_000 {
                let predicate =
                    builder.relation_exists(RelationExistsPolarityV1::Exists, relation, span);
                relation = builder.relation_filter(
                    builder.relation_source("r", span, None, span),
                    predicate,
                    span,
                );
            }
            drop(relation);
        }
        other => panic!("unknown raw HIR drop mode: {other}"),
    }
}

#[test]
fn direct_raw_hir_roots_drop_without_stack_abort() {
    for mode in ["relation", "document", "value", "bool", "shape", "mixed"] {
        assert_exact_child_passes("raw_deep_drop_child", "QGLANG_RAW_HIR_DROP_CHILD", mode);
    }
}

#[test]
fn extreme_owned_hir_child() {
    if std::env::var_os("QGLANG_DEEP_HIR_CHILD").is_none() {
        return;
    }
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let mut relation = builder.relation_source("r", span, None, span);
    for _ in 0..100_000 {
        relation = builder.relation_filter(relation, builder.bool_literal(true, span), span);
    }
    let document = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 1,
        items: vec![ProgramItemHirV1::from(builder.query(relation, None, span))],
    };
    assert!(matches!(
        check_tree(
            HirRootRefV1::Item(document.items.first().expect("one item")),
            1
        ),
        Err(HirIntegrityErrorV1::ExcessiveDepth {
            dimension: HirDepthDimensionV2::Relation,
            limit: 128,
            observed: 129,
            ..
        })
    ));
    let error = CheckedHirDocumentOwnedV2::try_new(document).expect_err("depth must reject");
    assert_eq!(error.code(), "hir.resource_limit_exceeded");

    let early_invalid =
        ProgramItemHirV1::from(builder.decl("x", span, builder.int("1", span), SpanV1::new(0, 2)));
    let mut relation = builder.relation_source("r", span, None, span);
    for _ in 0..100_000 {
        relation = builder.relation_filter(relation, builder.bool_literal(true, span), span);
    }
    let document = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 1,
        items: vec![
            early_invalid,
            ProgramItemHirV1::from(builder.query(relation, None, span)),
        ],
    };
    assert!(matches!(
        CheckedHirDocumentOwnedV2::try_new(document),
        Err(HirIntegrityErrorV1::SpanOutOfBounds { .. })
    ));

    let mut value = builder.int("1", span);
    for _ in 0..100_000 {
        value = builder.paren_value(value, span);
    }
    let document = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 1,
        items: vec![ProgramItemHirV1::from(builder.decl("x", span, value, span))],
    };
    assert!(matches!(
        CheckedHirDocumentOwnedV2::try_new(document),
        Err(HirIntegrityErrorV1::ExcessiveDepth {
            dimension: HirDepthDimensionV2::Expression,
            ..
        })
    ));

    let mut boolean = builder.bool_literal(true, span);
    for _ in 0..100_000 {
        boolean = builder.not(boolean, span);
    }
    let document = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 1,
        items: vec![ProgramItemHirV1::from(builder.rule(
            "rule",
            span,
            builder.int("1", span),
            boolean,
            span,
        ))],
    };
    assert!(matches!(
        CheckedHirDocumentOwnedV2::try_new(document),
        Err(HirIntegrityErrorV1::ExcessiveDepth {
            dimension: HirDepthDimensionV2::Expression,
            ..
        })
    ));

    let mut shape = builder.shape_leaf(builder.int("1", span), span);
    for _ in 0..100_000 {
        shape = builder.shape_list(shape, span);
    }
    let document = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 1,
        items: vec![ProgramItemHirV1::from(builder.query(
            builder.relation_source("r", span, None, span),
            Some(shape),
            span,
        ))],
    };
    assert!(matches!(
        CheckedHirDocumentOwnedV2::try_new(document),
        Err(HirIntegrityErrorV1::ExcessiveDepth {
            dimension: HirDepthDimensionV2::Shape,
            ..
        })
    ));
}

#[test]
fn extreme_owned_hir_rejects_without_abort() {
    assert_exact_child_passes("extreme_owned_hir_child", "QGLANG_DEEP_HIR_CHILD", "1");
}

#[test]
fn direct_hir_exact_and_one_over_depth_boundaries() {
    let builder = HirBuilderV1::new();
    let span = SpanV1::new(0, 1);
    let mut relation = builder.relation_source("r", span, None, span);
    for _ in 0..127 {
        relation = builder.relation_filter(relation, builder.bool_literal(true, span), span);
    }
    assert!(check_tree(HirRootRefV1::Relation(&relation), 1).is_ok());
    let relation = builder.relation_filter(relation, builder.bool_literal(true, span), span);
    assert!(matches!(
        check_tree(HirRootRefV1::Relation(&relation), 1),
        Err(HirIntegrityErrorV1::ExcessiveDepth {
            dimension: HirDepthDimensionV2::Relation,
            limit: 128,
            observed: 129,
            ..
        })
    ));

    let mut value = builder.int("1", span);
    for _ in 0..127 {
        value = builder.paren_value(value, span);
    }
    assert!(check_tree(HirRootRefV1::Value(&value), 1).is_ok());
    let value = builder.paren_value(value, span);
    assert!(matches!(
        check_tree(HirRootRefV1::Value(&value), 1),
        Err(HirIntegrityErrorV1::ExcessiveDepth {
            dimension: HirDepthDimensionV2::Expression,
            limit: 128,
            observed: 129,
            ..
        })
    ));

    let mut shape = builder.shape_leaf(builder.int("1", span), span);
    for _ in 0..63 {
        shape = builder.shape_list(shape, span);
    }
    assert!(check_tree(HirRootRefV1::Shape(&shape), 1).is_ok());
    let shape = builder.shape_list(shape, span);
    assert!(matches!(
        check_tree(HirRootRefV1::Shape(&shape), 1),
        Err(HirIntegrityErrorV1::ExcessiveDepth {
            dimension: HirDepthDimensionV2::Shape,
            limit: 64,
            observed: 65,
            ..
        })
    ));
}
