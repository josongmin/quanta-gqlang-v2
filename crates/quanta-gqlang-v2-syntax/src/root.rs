//! E01 named seam: the typed root discriminant of the parser.
//!
//! `RootKindV1` is the keyword-level identity of a parsed root. It mirrors the
//! 17 canonical root families (`SELECT`..`GATE` plus the two meta verbs) and the
//! composed [`crate::ast::AstRootV1::Composition`] grouping. The parser is
//! root-discriminated from `parse_document_v1` onward — there is no legacy
//! single-query/JSON carrier — so this enum is the stable handle other layers
//! pattern-match on without re-deriving the keyword from prose.
//!
//! The exhaustive `match` in [`RootKindV1::of_root_v1`] is the load-bearing
//! invariant: a new `AstRootV1` variant cannot compile until it is given a
//! `RootKindV1`, so the discriminant set can never silently drift from the AST.

use crate::ast::AstRootV1;

/// The typed root discriminant. Every parsed [`AstRootV1`] has exactly one
/// `RootKindV1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RootKindV1 {
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
    /// Interactive meta verbs (`USE PACK`, `REQUIRE CERTIFIED`).
    Meta,
    /// A `[LET]* <body> WITH <binding>` composition grouping.
    Composition,
}

impl RootKindV1 {
    /// The root discriminant of a parsed root. Exhaustive by construction: the
    /// `match` cannot compile if an `AstRootV1` variant lacks a mapping.
    #[must_use]
    pub const fn of_root_v1(root: &AstRootV1) -> Self {
        match root {
            AstRootV1::Select(_) => Self::Select,
            AstRootV1::Find(_) => Self::Find,
            AstRootV1::Check(_) => Self::Check,
            AstRootV1::Explain(_) => Self::Explain,
            AstRootV1::Show(_) => Self::Show,
            AstRootV1::Describe(_) => Self::Describe,
            AstRootV1::Sample(_) => Self::Sample,
            AstRootV1::Compare(_) => Self::Compare,
            AstRootV1::Export(_) => Self::Export,
            AstRootV1::View(_) => Self::View,
            AstRootV1::Rule(_) => Self::Rule,
            AstRootV1::Model(_) => Self::Model,
            AstRootV1::Pack(_) => Self::Pack,
            AstRootV1::Profile(_) => Self::Profile,
            AstRootV1::Gate(_) => Self::Gate,
            AstRootV1::Meta(_) => Self::Meta,
            AstRootV1::Composition(_) => Self::Composition,
        }
    }

    /// The canonical leading keyword for this kind, or `None` for
    /// [`RootKindV1::Composition`] (which has no keyword of its own — it is a
    /// grouping recognised by a trailing `WITH <binding>`).
    #[must_use]
    pub const fn keyword_v1(self) -> Option<&'static str> {
        match self {
            Self::Select => Some("SELECT"),
            Self::Find => Some("FIND"),
            Self::Check => Some("CHECK"),
            Self::Explain => Some("EXPLAIN"),
            Self::Show => Some("SHOW"),
            Self::Describe => Some("DESCRIBE"),
            Self::Sample => Some("SAMPLE"),
            Self::Compare => Some("COMPARE"),
            Self::Export => Some("EXPORT"),
            Self::View => Some("VIEW"),
            Self::Rule => Some("RULE"),
            Self::Model => Some("MODEL"),
            Self::Pack => Some("PACK"),
            Self::Profile => Some("PROFILE"),
            Self::Gate => Some("GATE"),
            // `Meta` covers two surface verbs (`USE`/`REQUIRE`); it has no single
            // canonical keyword, so callers must inspect the meta command.
            Self::Meta | Self::Composition => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "root-kind tests assert keyword presence for validated kinds"
    )]

    use super::RootKindV1;

    #[test]
    fn every_keyworded_kind_has_a_distinct_uppercase_keyword() {
        let keyworded = [
            RootKindV1::Select,
            RootKindV1::Find,
            RootKindV1::Check,
            RootKindV1::Explain,
            RootKindV1::Show,
            RootKindV1::Describe,
            RootKindV1::Sample,
            RootKindV1::Compare,
            RootKindV1::Export,
            RootKindV1::View,
            RootKindV1::Rule,
            RootKindV1::Model,
            RootKindV1::Pack,
            RootKindV1::Profile,
            RootKindV1::Gate,
        ];
        let mut seen = Vec::new();
        for kind in keyworded {
            let keyword = kind.keyword_v1().expect("keyworded kind has a keyword");
            assert_eq!(keyword, keyword.to_ascii_uppercase());
            assert!(!seen.contains(&keyword), "duplicate keyword {keyword}");
            seen.push(keyword);
        }
        assert_eq!(seen.len(), 15);
    }

    #[test]
    fn composition_has_no_leading_keyword() {
        assert_eq!(RootKindV1::Composition.keyword_v1(), None);
        assert_eq!(RootKindV1::Meta.keyword_v1(), None);
    }
}
