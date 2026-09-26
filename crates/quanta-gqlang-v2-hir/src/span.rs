//! Source spans for HIR nodes.
//!
//! Spans are byte-offset ranges into the original source text. They are the
//! source-mapping hook every HIR node carries so later tickets (binder, typer,
//! planner) can render diagnostics and explain trees against original syntax.

/// A half-open byte range `[start, end)` into the original source text.
///
/// Mirrors the syntax crate's `SpanV1` shape (byte offsets) so HIR construction
/// from syntax nodes is a structural copy, not a re-derivation. A span is valid
/// when `start <= end`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpanV1 {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

/// Stable identity of one source document within a frontend transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceIdV2(u32);

impl SourceIdV2 {
    /// Construct from the transaction-local raw identity.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw transaction-local identity.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// One source admitted to a frontend authority transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceRegistrationV2 {
    pub source_id: SourceIdV2,
    pub source_len: usize,
}

/// Checked source bounds. Entry-list construction keeps duplicate IDs
/// observable before a map can collapse them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRegistryViewV2 {
    lengths: std::collections::BTreeMap<SourceIdV2, usize>,
    identity: SourceRegistryIdentityV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceRegistryIdentityV2(std::sync::Arc<[SourceRegistrationV2]>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedSourceRegistryViewV2 {
    combined: SourceRegistryViewV2,
    parent_identity: SourceRegistryIdentityV2,
}

impl SourceRegistryViewV2 {
    pub fn try_new(entries: Vec<SourceRegistrationV2>) -> Result<Self, SourceRegistryDiagnosticV2> {
        let mut lengths = std::collections::BTreeMap::new();
        for entry in entries {
            if let Some(original_len) = lengths.insert(entry.source_id, entry.source_len) {
                return Err(SourceRegistryDiagnosticV2::DuplicateSourceId {
                    source_id: entry.source_id,
                    original_len,
                    duplicate_len: entry.source_len,
                });
            }
        }
        let registrations = lengths
            .iter()
            .map(|(source_id, source_len)| SourceRegistrationV2 {
                source_id: *source_id,
                source_len: *source_len,
            })
            .collect::<Vec<_>>();
        Ok(Self {
            lengths,
            identity: SourceRegistryIdentityV2(registrations.into()),
        })
    }

    pub fn try_check_location(
        &self,
        location: LocatedSpanV2,
    ) -> Result<CheckedLocatedSpanV2, SourceRegistryDiagnosticV2> {
        let source_len = self.lengths.get(&location.source_id).copied().ok_or(
            SourceRegistryDiagnosticV2::UnknownSourceId {
                source_id: location.source_id,
            },
        )?;
        location
            .validate(location.source_id, source_len)
            .map_err(|error| SourceRegistryDiagnosticV2::InvalidLocation { location, error })?;
        Ok(CheckedLocatedSpanV2 {
            location,
            registry_identity: self.identity(),
        })
    }

    pub fn try_extend_document(
        &self,
        entry: SourceRegistrationV2,
    ) -> Result<ExtendedSourceRegistryViewV2, SourceRegistryDiagnosticV2> {
        let mut entries = self
            .lengths
            .iter()
            .map(|(source_id, source_len)| SourceRegistrationV2 {
                source_id: *source_id,
                source_len: *source_len,
            })
            .collect::<Vec<_>>();
        entries.push(entry);
        Ok(ExtendedSourceRegistryViewV2 {
            combined: Self::try_new(entries)?,
            parent_identity: self.identity(),
        })
    }

    #[must_use]
    pub fn identity(&self) -> SourceRegistryIdentityV2 {
        self.identity.clone()
    }

    #[must_use]
    pub fn source_len(&self, source_id: SourceIdV2) -> Option<usize> {
        self.lengths.get(&source_id).copied()
    }
}

impl ExtendedSourceRegistryViewV2 {
    pub fn try_check_location(
        &self,
        location: LocatedSpanV2,
    ) -> Result<CheckedLocatedSpanV2, SourceRegistryDiagnosticV2> {
        self.combined.try_check_location(location)
    }
    #[must_use]
    pub fn parent_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.parent_identity
    }
    #[must_use]
    pub fn identity(&self) -> SourceRegistryIdentityV2 {
        self.combined.identity()
    }
    #[must_use]
    pub fn source_len(&self, source_id: SourceIdV2) -> Option<usize> {
        self.combined.source_len(source_id)
    }
    #[must_use]
    pub const fn combined(&self) -> &SourceRegistryViewV2 {
        &self.combined
    }
}

/// A location proven to be within the bounds of one exact source registry.
///
/// The fields are private so equality of raw source IDs cannot substitute for
/// the registry proof that minted the location.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CheckedLocatedSpanV2 {
    location: LocatedSpanV2,
    registry_identity: SourceRegistryIdentityV2,
}

impl CheckedLocatedSpanV2 {
    #[must_use]
    pub const fn location(&self) -> LocatedSpanV2 {
        self.location
    }

    #[must_use]
    pub const fn registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceRegistryDiagnosticV2 {
    DuplicateSourceId {
        source_id: SourceIdV2,
        original_len: usize,
        duplicate_len: usize,
    },
    UnknownSourceId {
        source_id: SourceIdV2,
    },
    InvalidLocation {
        location: LocatedSpanV2,
        error: LocatedSpanErrorV2,
    },
}

impl SourceRegistryDiagnosticV2 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::DuplicateSourceId { .. } => "hir.source_registry.duplicate_source_id",
            Self::UnknownSourceId { .. } => "hir.source_registry.unknown_source_id",
            Self::InvalidLocation {
                error: LocatedSpanErrorV2::Inverted { .. },
                ..
            } => "hir.source_registry.inverted_span",
            Self::InvalidLocation {
                error: LocatedSpanErrorV2::OutOfBounds { .. },
                ..
            } => "hir.source_registry.location_out_of_bounds",
            Self::InvalidLocation {
                error: LocatedSpanErrorV2::WrongSource { .. },
                ..
            } => "hir.source_registry.unknown_source_id",
        }
    }
}

impl core::fmt::Display for SourceRegistryDiagnosticV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DuplicateSourceId {
                source_id,
                original_len,
                duplicate_len,
            } => write!(
                f,
                "{}: source {} registered twice with lengths {} and {}",
                self.code(),
                source_id.raw(),
                original_len,
                duplicate_len
            ),
            Self::UnknownSourceId { source_id } => write!(
                f,
                "{}: source {} is not registered",
                self.code(),
                source_id.raw()
            ),
            Self::InvalidLocation { location, .. } => write!(
                f,
                "{}: span {}..{} is invalid for source {}",
                self.code(),
                location.span.start,
                location.span.end,
                location.source_id.raw()
            ),
        }
    }
}
impl std::error::Error for SourceRegistryDiagnosticV2 {}

/// A byte span tied to the source document whose bytes it indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocatedSpanV2 {
    /// Owning source identity.
    pub source_id: SourceIdV2,
    /// Byte range in that source.
    pub span: SpanV1,
}

impl LocatedSpanV2 {
    /// Validate source identity and byte bounds against a registered source.
    pub fn validate(
        self,
        expected_source: SourceIdV2,
        source_len: usize,
    ) -> Result<(), LocatedSpanErrorV2> {
        if self.source_id != expected_source {
            return Err(LocatedSpanErrorV2::WrongSource {
                expected: expected_source,
                actual: self.source_id,
            });
        }
        if !self.span.is_well_formed() {
            return Err(LocatedSpanErrorV2::Inverted { span: self.span });
        }
        if self.span.end > source_len {
            return Err(LocatedSpanErrorV2::OutOfBounds {
                span: self.span,
                source_len,
            });
        }
        Ok(())
    }
}

/// Failure to validate a located span against a registered source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocatedSpanErrorV2 {
    /// The span points at another source.
    WrongSource {
        /// Registered source.
        expected: SourceIdV2,
        /// Carried source.
        actual: SourceIdV2,
    },
    /// The byte range is inverted.
    Inverted {
        /// Invalid span.
        span: SpanV1,
    },
    /// The byte range extends beyond the registered source.
    OutOfBounds {
        /// Invalid span.
        span: SpanV1,
        /// Registered source byte length.
        source_len: usize,
    },
}

impl SpanV1 {
    /// Construct a span from raw byte offsets without validation.
    ///
    /// Use [`SpanV1::checked`] when the offsets are untrusted; integrity
    /// checking ([`crate::integrity`]) will also reject inverted spans.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Construct a span, returning `None` when `start > end`.
    #[must_use]
    pub const fn checked(start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// A zero-width span at `offset`. Useful for synthesized nodes.
    #[must_use]
    pub const fn point(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Byte length of the span.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Whether the span covers zero bytes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Whether `start <= end`. Inverted spans are an integrity violation.
    #[must_use]
    pub const fn is_well_formed(&self) -> bool {
        self.start <= self.end
    }

    /// Whether this span is fully contained within `[0, source_len)` bounds,
    /// i.e. `end <= source_len` and the span is well formed.
    #[must_use]
    pub const fn is_within(&self, source_len: usize) -> bool {
        self.is_well_formed() && self.end <= source_len
    }

    /// Smallest span covering both `self` and `other`.
    #[must_use]
    pub fn merge(&self, other: &SpanV1) -> SpanV1 {
        SpanV1 {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "owner assertions")]
mod source_registry_tests {
    use super::*;
    #[test]
    fn registry_rejects_duplicates_unknown_inverted_and_oob_with_exact_codes() {
        let source = SourceIdV2::from_raw(1);
        let duplicate = SourceRegistryViewV2::try_new(vec![
            SourceRegistrationV2 {
                source_id: source,
                source_len: 4,
            },
            SourceRegistrationV2 {
                source_id: source,
                source_len: 5,
            },
        ])
        .unwrap_err();
        assert_eq!(duplicate.code(), "hir.source_registry.duplicate_source_id");
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id: source,
            source_len: 4,
        }])
        .unwrap();
        assert_eq!(
            registry
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(2),
                    span: SpanV1::point(0)
                })
                .unwrap_err()
                .code(),
            "hir.source_registry.unknown_source_id"
        );
        assert_eq!(
            registry
                .try_check_location(LocatedSpanV2 {
                    source_id: source,
                    span: SpanV1::new(3, 2)
                })
                .unwrap_err()
                .code(),
            "hir.source_registry.inverted_span"
        );
        assert_eq!(
            registry
                .try_check_location(LocatedSpanV2 {
                    source_id: source,
                    span: SpanV1::new(0, 5)
                })
                .unwrap_err()
                .code(),
            "hir.source_registry.location_out_of_bounds"
        );
    }
    #[test]
    fn registry_identity_is_deterministic_and_copy_extend_rejects_collision() {
        let one = SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 4,
        };
        let two = SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(2),
            source_len: 8,
        };
        let a = SourceRegistryViewV2::try_new(vec![one, two]).unwrap();
        let b = SourceRegistryViewV2::try_new(vec![two, one]).unwrap();
        assert_eq!(a.identity(), b.identity());
        assert_eq!(a.source_len(one.source_id), Some(4));
        assert_eq!(a.source_len(two.source_id), Some(8));
        assert_eq!(a.source_len(SourceIdV2::from_raw(99)), None);
        assert_eq!(
            a.try_extend_document(one).unwrap_err().code(),
            "hir.source_registry.duplicate_source_id"
        );
        let extended = a
            .try_extend_document(SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(3),
                source_len: 12,
            })
            .unwrap();
        assert_eq!(extended.parent_identity(), &a.identity());
        assert_ne!(extended.identity(), a.identity());
        assert_eq!(extended.source_len(SourceIdV2::from_raw(3)), Some(12));
        assert_eq!(extended.source_len(SourceIdV2::from_raw(99)), None);
    }
    #[test]
    fn identity_is_structural_and_checked_locations_share_backing() {
        let entries = vec![SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 20,
        }];
        let first = SourceRegistryViewV2::try_new(entries.clone()).unwrap();
        let equal = SourceRegistryViewV2::try_new(entries).unwrap();
        let unequal = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 21,
        }])
        .unwrap();
        assert_eq!(first.identity(), equal.identity());
        assert_ne!(first.identity(), unequal.identity());
        let left = first
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(0, 1),
            })
            .unwrap();
        let right = first
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(1, 2),
            })
            .unwrap();
        assert_eq!(
            left.location(),
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(0, 1),
            }
        );
        assert_eq!(
            right.location(),
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(1, 2),
            }
        );
        assert!(std::sync::Arc::ptr_eq(
            &left.registry_identity.0,
            &right.registry_identity.0
        ));
    }
}
