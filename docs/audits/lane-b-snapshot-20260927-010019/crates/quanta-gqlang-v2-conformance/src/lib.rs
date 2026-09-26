#![forbid(unsafe_code)]
//! Facade-only conformance registry for every public source family.

macro_rules! declare_root_families_v2 {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum RootFamilyV2 { $($variant),+ }
        pub const ROOT_FAMILIES_V2: &[RootFamilyV2] = &[$(RootFamilyV2::$variant),+];
    };
}

declare_root_families_v2!(
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
    RuleBinding,
    Model,
    Pack,
    Profile,
    Gate,
    UsePack,
    RequireCertified,
    Composition,
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpectedDispositionV2 {
    Planned {
        kind: &'static str,
        result_kind: &'static str,
        effect: &'static str,
    },
    NoPlan {
        reason: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootFamilySpecV2 {
    pub family: RootFamilyV2,
    pub keyword: &'static str,
    pub source: &'static str,
    pub expected: ExpectedDispositionV2,
}

const fn planned(
    kind: &'static str,
    result_kind: &'static str,
    effect: &'static str,
) -> ExpectedDispositionV2 {
    ExpectedDispositionV2::Planned {
        kind,
        result_kind,
        effect,
    }
}

const fn no_plan(reason: &'static str) -> ExpectedDispositionV2 {
    ExpectedDispositionV2::NoPlan { reason }
}

#[must_use]
pub const fn spec_for_v2(family: RootFamilyV2) -> RootFamilySpecV2 {
    match family {
        RootFamilyV2::Select => RootFamilySpecV2 {
            family,
            keyword: "from",
            source: "from users u | where u.id > 1 | project u.id as id, u.name as name",
            expected: planned("relation", "table", "read_only"),
        },
        RootFamilyV2::Find => RootFamilySpecV2 {
            family,
            keyword: "find",
            source: "find taint api.entry -> db.write | sanitizers=authz.clean | depth=3",
            expected: planned("graph", "path_witness", "proof_evidence_only"),
        },
        RootFamilyV2::Check => RootFamilySpecV2 {
            family,
            keyword: "check",
            source: "check rule.hybrid | scope=repo | pattern=pattern.hybrid | confirm=require | emit=sarif | mode=hybrid using find reachability src.entry -> sink.write | depth=8",
            expected: planned("check", "alert", "export_only"),
        },
        RootFamilyV2::Explain => RootFamilySpecV2 {
            family,
            keyword: "explain",
            source: "explain rule.sqli | evidence=trace | truth=confirm",
            expected: planned("presentation", "proof_explain", "read_only"),
        },
        RootFamilyV2::Show => RootFamilySpecV2 {
            family,
            keyword: "show",
            source: "show capabilities",
            expected: planned("presentation", "table", "read_only"),
        },
        RootFamilyV2::Describe => RootFamilySpecV2 {
            family,
            keyword: "describe",
            source: "describe rule::owasp.sqli",
            expected: planned("presentation", "alert", "read_only"),
        },
        RootFamilyV2::Sample => RootFamilySpecV2 {
            family,
            keyword: "sample",
            source: "sample query=\"callgraph\" | mode=semantic | limit=5 | scope=repo",
            expected: planned("presentation", "table", "read_only"),
        },
        RootFamilyV2::Compare => RootFamilySpecV2 {
            family,
            keyword: "compare",
            source: "compare baseline=run.a | candidate=run.b | metric=semantic",
            expected: planned("presentation", "path_witness", "read_only"),
        },
        RootFamilyV2::Export => RootFamilySpecV2 {
            family,
            keyword: "export",
            source: "export source=result.latest | kind=sarif",
            expected: planned("presentation", "table", "export_only"),
        },
        RootFamilyV2::View => RootFamilySpecV2 {
            family,
            keyword: "view",
            source: "view catalog.users | query-ref=queries.catalog.users",
            expected: no_plan("catalog_declaration"),
        },
        RootFamilyV2::RuleBinding => RootFamilySpecV2 {
            family,
            keyword: "ruledef",
            source: "ruledef shell.exec | check-ref=checks.shell-exec",
            expected: no_plan("catalog_declaration"),
        },
        RootFamilyV2::Model => RootFamilySpecV2 {
            family,
            keyword: "model",
            source: "model embeddings.code | provider=openai | source=models.code",
            expected: no_plan("catalog_declaration"),
        },
        RootFamilyV2::Pack => RootFamilySpecV2 {
            family,
            keyword: "pack",
            source: "pack secure-pack | source=registry.security | activation=workspace",
            expected: no_plan("catalog_declaration"),
        },
        RootFamilyV2::Profile => RootFamilySpecV2 {
            family,
            keyword: "profile",
            source: "profile strict | capabilities=graph.read,graph.trace | admission=certified-only",
            expected: no_plan("catalog_declaration"),
        },
        RootFamilyV2::Gate => RootFamilySpecV2 {
            family,
            keyword: "gate",
            source: "gate release | checks=owasp.sql,authz.path | binding=all",
            expected: no_plan("catalog_declaration"),
        },
        RootFamilyV2::UsePack => RootFamilySpecV2 {
            family,
            keyword: "use",
            source: "use pack secure-pack | activation=session | lockfile=packs.secure-pack.lock",
            expected: no_plan("meta_activation"),
        },
        RootFamilyV2::RequireCertified => RootFamilySpecV2 {
            family,
            keyword: "require",
            source: "require certified",
            expected: planned("meta_require_certified", "proof_explain", "proof_evidence_only"),
        },
        RootFamilyV2::Composition => RootFamilySpecV2 {
            family,
            keyword: "let",
            source: "let suspects = find taint a.b -> c.d | depth=2 find taint api.entry -> db.write | depth=3 with suspects",
            expected: planned("composition", "path_witness", "proof_evidence_only"),
        },
    }
}

#[must_use]
pub fn root_family_specs_v2() -> impl ExactSizeIterator<Item = RootFamilySpecV2> {
    ROOT_FAMILIES_V2.iter().copied().map(spec_for_v2)
}

pub const SUPPLEMENTAL_SOURCES_V2: [&str; 2] = ["let x = 1 + 2", "rule adult : x if x = 1"];

pub const UNSUPPORTED_SOURCE_V2: &str = "totally not a qglang root";
