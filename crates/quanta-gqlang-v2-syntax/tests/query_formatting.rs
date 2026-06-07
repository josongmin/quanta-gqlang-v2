use quanta_gqlang_v2_syntax::ast::{AstRootV1, DocumentAstV1};
use quanta_gqlang_v2_syntax::declarations::pack::{GateDeclAstV1, PackDeclAstV1, ProfileDeclAstV1};
use quanta_gqlang_v2_syntax::declarations::view::{ModelDeclAstV1, RuleDeclAstV1, ViewDeclAstV1};
use quanta_gqlang_v2_syntax::format_document_v1;
use quanta_gqlang_v2_syntax::interactive::meta::MetaCommandAstV1;
use quanta_gqlang_v2_syntax::interactive::sample::{
    CompareIntentAstV1, CompareMetricV1, InteractiveRefAstV1, SampleRetrievalIntentAstV1,
    SampleRetrievalModeV1,
};
use quanta_gqlang_v2_syntax::parse_document_v1;
use quanta_gqlang_v2_syntax::roots::check::{
    CheckConfirmIntentV1, CheckRootAstV1, CheckRootIntentV1,
};

fn assert_input_round_trip_v1(input: &str, expected: &str) {
    let parsed = parse_document_v1(input).expect("input must parse");
    assert_formatted_doc_round_trip_v1(&parsed, expected);
}

fn assert_formatted_doc_round_trip_v1(doc: &DocumentAstV1, expected: &str) {
    let formatted = format_document_v1(doc);
    assert_eq!(formatted, expected);

    let reparsed = parse_document_v1(&formatted).expect("formatted output must parse");
    assert_eq!(format_document_v1(&reparsed), expected);
    assert_eq!(
        reparsed,
        parse_document_v1(expected).expect("expected must parse")
    );
}

#[test]
fn formats_core_roots_idempotently_v1() {
    assert_input_round_trip_v1(
        " SELECT  id , path FROM catalog.files WHERE severity = 'high' LIMIT 10 ; ",
        "SELECT id, path FROM catalog.files WHERE severity = 'high' LIMIT 10",
    );
    assert_input_round_trip_v1(
        "FIND taint user.input -> shell.exec | sanitizers=escape,quote | depth=4",
        "FIND taint user.input -> shell.exec | sanitizers=escape,quote | depth=4",
    );
    assert_input_round_trip_v1(
        "EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth_expectation=confirm",
        "EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm",
    );

    let check_intent = CheckRootIntentV1::new(
        "rule.security/no-shell",
        "security.review",
        Some("repo:semantica|crate=runtime".to_string()),
        Some("path ~= \"src/**/*.rs\"".to_string()),
        CheckConfirmIntentV1::Require,
    )
    .expect("check intent");
    let check_root = CheckRootAstV1::from_intent(check_intent.clone());
    let expected = format!("CHECK {} {}", check_root.rule_id, check_root.scope);
    let check_doc = DocumentAstV1 {
        roots: vec![AstRootV1::Check(check_root)],
        with_clause: None,
    };
    assert_formatted_doc_round_trip_v1(&check_doc, &expected);
}

#[test]
fn formats_interactive_roots_idempotently_v1() {
    assert_input_round_trip_v1("SHOW capability", "SHOW capabilities");
    assert_input_round_trip_v1(
        "DESCRIBE profiles::strict.review",
        "DESCRIBE profile::strict.review",
    );

    let sample_doc = DocumentAstV1 {
        roots: vec![AstRootV1::Sample(
            quanta_gqlang_v2_syntax::interactive::sample::SampleRootAstV1::from_intent(
                SampleRetrievalIntentAstV1 {
                    query: "error propagation".to_string(),
                    scope: Some(InteractiveRefAstV1 {
                        kind: "repo".to_string(),
                        locator: "frontend".to_string(),
                    }),
                    mode: SampleRetrievalModeV1::Hybrid,
                    limit: 5,
                },
            ),
        )],
        with_clause: None,
    };
    assert_formatted_doc_round_trip_v1(
        &sample_doc,
        "SAMPLE query=error propagation ;; mode=hybrid ;; limit=5 ;; scope=repo::frontend",
    );

    let compare_doc = DocumentAstV1 {
        roots: vec![AstRootV1::Compare(
            quanta_gqlang_v2_syntax::interactive::sample::CompareRootAstV1::from_intent(
                CompareIntentAstV1 {
                    baseline: InteractiveRefAstV1 {
                        kind: "query".to_string(),
                        locator: "baseline".to_string(),
                    },
                    candidate: InteractiveRefAstV1 {
                        kind: "query".to_string(),
                        locator: "candidate".to_string(),
                    },
                    metric: CompareMetricV1::Semantic,
                },
            ),
        )],
        with_clause: None,
    };
    assert_formatted_doc_round_trip_v1(
        &compare_doc,
        "COMPARE baseline=query::baseline ;; candidate=query::candidate ;; metric=semantic",
    );

    assert_input_round_trip_v1(
        "EXPORT source=query::baseline ;; kind=graphml ;; profile=security-review",
        "EXPORT source=query::baseline ;; kind=graph-ml ;; profile=security-review",
    );
}

#[test]
fn formats_declarations_and_meta_idempotently_v1() {
    assert_input_round_trip_v1(
        "VIEW catalog.users ;; query-ref=queries/catalog/users",
        "VIEW catalog.users ;; query-ref=queries/catalog/users",
    );
    assert_input_round_trip_v1(
        "VIEW recent.errors ;; body=SELECT id, severity FROM catalog.errors WHERE active = true LIMIT 25",
        "VIEW recent.errors ;; body=SELECT id, severity FROM catalog.errors WHERE active = true LIMIT 25",
    );
    assert_input_round_trip_v1(
        "RULE shell.exec ;; check-ref=checks/security/shell-exec",
        "RULE shell.exec ;; check-ref=checks/security/shell-exec",
    );
    assert_input_round_trip_v1(
        "MODEL embeddings.code ;; provider=openai ;; source=models/code-search",
        "MODEL embeddings.code ;; provider=openai ;; source=models/code-search",
    );
    assert_input_round_trip_v1(
        "PACK secure-pack ;; source=registry://packs/secure ;; activation=WORKSPACE",
        "PACK secure-pack ;; source=registry://packs/secure ;; activation=workspace",
    );
    assert_input_round_trip_v1(
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=review_required",
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=review-required",
    );
    assert_input_round_trip_v1(
        "GATE release.blocker ;; checks=rule.alpha,rule.beta ;; binding=all",
        "GATE release.blocker ;; checks=rule.alpha,rule.beta ;; binding=all",
    );
    assert_input_round_trip_v1(
        "USE PACK secure-pack ;; activation=WORKSPACE ;; lockfile=packs/secure-pack.lock",
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=packs/secure-pack.lock",
    );
    assert_input_round_trip_v1("REQUIRE CERTIFIED", "REQUIRE CERTIFIED");
}

#[test]
fn formatted_output_is_deterministic_for_manual_document_roots_v1() {
    let doc = DocumentAstV1 {
        roots: vec![
            AstRootV1::Meta(MetaCommandAstV1::UsePack {
                pack_id: "secure-pack".to_string(),
                activation_mode: Some("session".to_string()),
                lockfile_path: Some("packs/secure-pack.lock".to_string()),
            }),
            AstRootV1::Pack(PackDeclAstV1 {
                name: "secure-pack ;; source=registry://packs/secure ;; activation=session"
                    .to_string(),
            }),
            AstRootV1::Profile(ProfileDeclAstV1 {
                name: "strict-review ;; capabilities=graph.read,graph.trace ;; admission=allowlist"
                    .to_string(),
            }),
            AstRootV1::Gate(GateDeclAstV1 {
                name: "release.blocker ;; checks=rule.alpha,rule.beta ;; binding=any".to_string(),
            }),
            AstRootV1::View(ViewDeclAstV1 {
                name: "catalog.users ;; query-ref=queries/catalog/users".to_string(),
            }),
            AstRootV1::Rule(RuleDeclAstV1 {
                name: "shell.exec ;; check-ref=checks/security/shell-exec".to_string(),
            }),
            AstRootV1::Model(ModelDeclAstV1 {
                name: "embeddings.code ;; provider=openai ;; source=models/code-search".to_string(),
            }),
        ],
        with_clause: None,
    };

    assert_eq!(
        format_document_v1(&doc),
        [
            "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock",
            "PACK secure-pack ;; source=registry://packs/secure ;; activation=session",
            "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=allowlist",
            "GATE release.blocker ;; checks=rule.alpha,rule.beta ;; binding=any",
            "VIEW catalog.users ;; query-ref=queries/catalog/users",
            "RULE shell.exec ;; check-ref=checks/security/shell-exec",
            "MODEL embeddings.code ;; provider=openai ;; source=models/code-search",
        ]
        .join("\n")
    );
}
