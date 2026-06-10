#![forbid(unsafe_code)]
//! Dev-only conformance registry — the single source of truth for the 17 DSL
//! root families' *front-door breadth*.
//!
//! This crate ships no production behavior (`publish = false`); it exists so the
//! CLI/LSP negative hellgates, the binary example matrix, and the doc-conformance
//! gates all iterate ONE table instead of each maintaining its own per-family
//! list that can silently drift out of sync.
//!
//! Scope discipline: this registry is breadth (one canonical example + one
//! canonical fail-closed negative per family). The in-crate syntax/sema
//! conformance corpora remain the DEPTH corpora (many cases per family) and are
//! deliberately NOT folded in here.
//!
//! Construction is a `match` over [`RootFamilyV1`] in [`spec_for_v1`], so adding
//! a family variant is a compile error until every field is supplied. The
//! `tests/self_validation.rs` hellgate then proves — through the REAL
//! parse → sema → lower pipeline — that every example lowers to its declared IR
//! root kind and every negative fails closed with its declared code and message
//! fragment. The negative's *display class* (`syntax` / `sema`) is not stored
//! here: it is derived from the code via the single ir authority
//! [`quanta_gqlang_v2_ir::diagnostic_class::front_door_diagnostic_class_v1`], so
//! there is no second copy of that mapping to drift.

pub mod leakage;

use quanta_gqlang_v2_ir::planning::PublicPlanRootKindV1;

/// Declare the closed `RootFamilyV1` enum AND the canonical `ROOT_FAMILIES_V1`
/// list from ONE source list, so the enum and the iteration list can never
/// drift: adding a family is a single edit to the invocation below, and
/// `spec_for_v1`'s exhaustive `match` then refuses to compile until its spec is
/// supplied. (Rust has no built-in variant iteration; this macro is the
/// no-dependency way to keep the single enumeration honest.)
macro_rules! declare_root_families_v1 {
    ($($variant:ident),+ $(,)?) => {
        /// The closed DSL root families. Closed-world by design: extension is a
        /// new variant plus the compiler enumerating every site that must answer.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum RootFamilyV1 {
            $($variant),+
        }

        /// Every family, in canonical order — generated from the same list as the
        /// enum, so it always covers exactly the variant set.
        pub const ROOT_FAMILIES_V1: &[RootFamilyV1] = &[$(RootFamilyV1::$variant),+];
    };
}

declare_root_families_v1!(
    Select,
    Find,
    Check,
    Explain,
    Show,
    Describe,
    Sample,
    Compare,
    Export,
    View,
    Rule,
    Model,
    Pack,
    Profile,
    Gate,
    UsePack,
    RequireCertified,
    Composition,
);

/// One canonical fail-closed negative for a family: the malformed source, the
/// exact diagnostic code, and a distinctive message fragment (the fragment
/// guards against a same-code-wrong-reason regression the bare code cannot
/// catch). The display class is derived from `code`, not stored.
#[derive(Clone, Copy, Debug)]
pub struct NegativeCaseV1 {
    pub source: &'static str,
    pub code: &'static str,
    pub message_fragment: &'static str,
}

/// Per-command front-door applicability. Carried here so command matrices read
/// applicability from the single source of truth instead of each test growing a
/// local "rows this command skips" denylist.
#[derive(Clone, Copy, Debug)]
pub struct CommandApplicabilityV1 {
    /// The CLI `explain` command / LSP `explain` method accept only EXPLAIN and
    /// FIND first roots.
    pub explain: bool,
}

/// The full front-door spec for one root family.
#[derive(Clone, Copy, Debug)]
pub struct RootSurfaceSpecV1 {
    pub family: RootFamilyV1,
    pub keyword: &'static str,
    pub example_path: &'static str,
    pub ir_root_kind: PublicPlanRootKindV1,
    pub negative: NegativeCaseV1,
    pub applicability: CommandApplicabilityV1,
}

const fn negative_v1(
    source: &'static str,
    code: &'static str,
    message_fragment: &'static str,
) -> NegativeCaseV1 {
    NegativeCaseV1 {
        source,
        code,
        message_fragment,
    }
}

/// The single per-family authority. Exhaustive `match`: a new [`RootFamilyV1`]
/// variant cannot compile until its spec is supplied here.
#[must_use]
pub const fn spec_for_v1(family: RootFamilyV1) -> RootSurfaceSpecV1 {
    use PublicPlanRootKindV1 as Kind;

    let core = CommandApplicabilityV1 { explain: false };
    let explainable = CommandApplicabilityV1 { explain: true };

    match family {
        RootFamilyV1::Select => RootSurfaceSpecV1 {
            family,
            keyword: "SELECT",
            example_path: "examples/select/basic.qglang",
            ir_root_kind: Kind::Select,
            negative: negative_v1(
                "SELECT id catalog.users",
                "expected_from_keyword",
                "must be followed by FROM",
            ),
            applicability: core,
        },
        RootFamilyV1::Find => RootSurfaceSpecV1 {
            family,
            keyword: "FIND",
            example_path: "examples/find/taint.qglang",
            ir_root_kind: Kind::Find,
            negative: negative_v1(
                "FIND unknown api.entry -> db.write",
                "unsupported_find_subkind",
                "unsupported FIND subkind",
            ),
            applicability: explainable,
        },
        RootFamilyV1::Check => RootSurfaceSpecV1 {
            family,
            keyword: "CHECK",
            example_path: "examples/check/hybrid_rule.qglang",
            ir_root_kind: Kind::Check,
            negative: negative_v1(
                "CHECK rule.hybrid",
                "check_missing_rule_or_scope",
                "CHECK requires",
            ),
            applicability: core,
        },
        RootFamilyV1::Explain => RootSurfaceSpecV1 {
            family,
            keyword: "EXPLAIN",
            example_path: "examples/explain/why_no_result.qglang",
            ir_root_kind: Kind::Explain,
            negative: negative_v1(
                "EXPLAIN rule::owasp.sql-injection ;; truth=maybe",
                "invalid_explain_shape",
                "truth expectation",
            ),
            applicability: explainable,
        },
        RootFamilyV1::Show => RootSurfaceSpecV1 {
            family,
            keyword: "SHOW",
            example_path: "examples/show/capabilities.qglang",
            ir_root_kind: Kind::Interactive,
            negative: negative_v1("SHOW catalog", "unsupported_show_category", "SHOW category"),
            applicability: core,
        },
        RootFamilyV1::Describe => RootSurfaceSpecV1 {
            family,
            keyword: "DESCRIBE",
            example_path: "examples/describe/rule_detail.qglang",
            ir_root_kind: Kind::Interactive,
            negative: negative_v1(
                "DESCRIBE model::",
                "describe_target_invalid_shape",
                "DESCRIBE target must use",
            ),
            applicability: core,
        },
        RootFamilyV1::Sample => RootSurfaceSpecV1 {
            family,
            keyword: "SAMPLE",
            example_path: "examples/sample/retrieval.qglang",
            ir_root_kind: Kind::Retrieval,
            negative: negative_v1(
                "SAMPLE",
                "sample_missing_query",
                "requires structured payload",
            ),
            applicability: core,
        },
        RootFamilyV1::Compare => RootSurfaceSpecV1 {
            family,
            keyword: "COMPARE",
            example_path: "examples/compare/semantic_delta.qglang",
            ir_root_kind: Kind::Interactive,
            negative: negative_v1(
                "COMPARE",
                "compare_missing_baseline",
                "requires structured payload",
            ),
            applicability: core,
        },
        RootFamilyV1::Export => RootSurfaceSpecV1 {
            family,
            keyword: "EXPORT",
            example_path: "examples/export/sarif.qglang",
            ir_root_kind: Kind::Interactive,
            negative: negative_v1(
                "EXPORT",
                "export_missing_source",
                "requires structured payload",
            ),
            applicability: core,
        },
        RootFamilyV1::View => RootSurfaceSpecV1 {
            family,
            keyword: "VIEW",
            example_path: "examples/view/catalog_users.qglang",
            ir_root_kind: Kind::Declaration,
            negative: negative_v1("VIEW", "view_missing_name", "requires payload"),
            applicability: core,
        },
        RootFamilyV1::Rule => RootSurfaceSpecV1 {
            family,
            keyword: "RULE",
            example_path: "examples/rule/shell_exec.qglang",
            ir_root_kind: Kind::Declaration,
            negative: negative_v1("RULE", "rule_missing_name", "requires payload"),
            applicability: core,
        },
        RootFamilyV1::Model => RootSurfaceSpecV1 {
            family,
            keyword: "MODEL",
            example_path: "examples/model/embeddings_code.qglang",
            ir_root_kind: Kind::Declaration,
            negative: negative_v1("MODEL", "model_missing_name", "requires payload"),
            applicability: core,
        },
        RootFamilyV1::Pack => RootSurfaceSpecV1 {
            family,
            keyword: "PACK",
            example_path: "examples/pack/security_default.qglang",
            ir_root_kind: Kind::Declaration,
            negative: negative_v1("PACK", "pack_missing_name", "requires payload"),
            applicability: core,
        },
        RootFamilyV1::Profile => RootSurfaceSpecV1 {
            family,
            keyword: "PROFILE",
            example_path: "examples/profile/strict_review.qglang",
            ir_root_kind: Kind::Declaration,
            negative: negative_v1("PROFILE", "profile_missing_name", "requires payload"),
            applicability: core,
        },
        RootFamilyV1::Gate => RootSurfaceSpecV1 {
            family,
            keyword: "GATE",
            example_path: "examples/gate/release_gate.qglang",
            ir_root_kind: Kind::Declaration,
            negative: negative_v1("GATE", "gate_missing_name", "requires payload"),
            applicability: core,
        },
        RootFamilyV1::UsePack => RootSurfaceSpecV1 {
            family,
            keyword: "USE PACK",
            example_path: "examples/use_pack/secure_pack.qglang",
            ir_root_kind: Kind::Meta,
            negative: negative_v1(
                "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.txt",
                "use_pack_invalid_lockfile",
                "must end with `.lock`",
            ),
            applicability: core,
        },
        RootFamilyV1::RequireCertified => RootSurfaceSpecV1 {
            family,
            keyword: "REQUIRE CERTIFIED",
            example_path: "examples/require_certified/strict_only.qglang",
            ir_root_kind: Kind::Meta,
            negative: negative_v1(
                "REQUIRE CERTIFIED ;; truth=maybe",
                "unsupported_require_meta",
                "unsupported REQUIRE target",
            ),
            applicability: core,
        },
        // `LET <name> = <producer>` bindings consumed by a body that names them
        // via a trailing `WITH <binding>` (E03C). The leading token is LET.
        RootFamilyV1::Composition => RootSurfaceSpecV1 {
            family,
            keyword: "LET",
            example_path: "examples/composition/let_with_find.qglang",
            ir_root_kind: Kind::Composition,
            negative: negative_v1(
                "LET a = SELECT id FROM catalog.users",
                "composition_unconsumed_let",
                "must be consumed by a body",
            ),
            applicability: core,
        },
    }
}

/// The full registry, one spec per family in [`ROOT_FAMILIES_V1`] order.
#[must_use]
pub fn root_surface_specs_v1() -> Vec<RootSurfaceSpecV1> {
    ROOT_FAMILIES_V1.iter().copied().map(spec_for_v1).collect()
}
