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

/// Checked source bounds. Up to two registrations stay inline; larger host
/// registries share one sorted owner across cloned views and one-document
/// extensions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRegistryViewV2 {
    identity: SourceRegistryIdentityV2,
}

#[derive(Clone, Debug)]
pub struct SourceRegistryIdentityV2(SourceRegistrationsV2);

#[derive(Clone, Debug)]
enum SourceRegistrationsV2 {
    Empty,
    Single(SourceRegistrationV2),
    Pair([SourceRegistrationV2; 2]),
    Many(std::sync::Arc<Vec<SourceRegistrationV2>>),
    ExtendedMany {
        parent: std::sync::Arc<Vec<SourceRegistrationV2>>,
        document: SourceRegistrationV2,
    },
}

impl SourceRegistryIdentityV2 {
    fn registrations_v2(&self) -> SourceRegistrationIterV2<'_> {
        match &self.0 {
            SourceRegistrationsV2::Empty => SourceRegistrationIterV2::new(&[], None),
            SourceRegistrationsV2::Single(entry) => {
                SourceRegistrationIterV2::new(std::slice::from_ref(entry), None)
            }
            SourceRegistrationsV2::Pair(entries) => SourceRegistrationIterV2::new(entries, None),
            SourceRegistrationsV2::Many(entries) => SourceRegistrationIterV2::new(entries, None),
            SourceRegistrationsV2::ExtendedMany { parent, document } => {
                SourceRegistrationIterV2::new(parent, Some(*document))
            }
        }
    }

    fn source_len_v2(&self, source_id: SourceIdV2) -> Option<usize> {
        if let SourceRegistrationsV2::ExtendedMany { parent, document } = &self.0 {
            if document.source_id == source_id {
                return Some(document.source_len);
            }
            return parent
                .binary_search_by_key(&source_id, |entry| entry.source_id)
                .ok()
                .and_then(|index| parent.get(index))
                .map(|entry| entry.source_len);
        }
        let entries = match &self.0 {
            SourceRegistrationsV2::Empty => &[][..],
            SourceRegistrationsV2::Single(entry) => std::slice::from_ref(entry),
            SourceRegistrationsV2::Pair(entries) => entries.as_slice(),
            SourceRegistrationsV2::Many(entries) => entries.as_slice(),
            SourceRegistrationsV2::ExtendedMany { .. } => unreachable!(),
        };
        entries
            .binary_search_by_key(&source_id, |entry| entry.source_id)
            .ok()
            .and_then(|index| entries.get(index))
            .map(|entry| entry.source_len)
    }
}

/// Walk one canonical sorted registry without materializing a merged Vec.
struct SourceRegistrationIterV2<'a> {
    base: &'a [SourceRegistrationV2],
    document: Option<SourceRegistrationV2>,
    index: usize,
}

impl<'a> SourceRegistrationIterV2<'a> {
    const fn new(base: &'a [SourceRegistrationV2], document: Option<SourceRegistrationV2>) -> Self {
        Self {
            base,
            document,
            index: 0,
        }
    }
}

impl Iterator for SourceRegistrationIterV2<'_> {
    type Item = SourceRegistrationV2;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(document) = self.document {
            if self
                .base
                .get(self.index)
                .map_or(true, |entry| document.source_id < entry.source_id)
            {
                self.document = None;
                return Some(document);
            }
        }
        let entry = self.base.get(self.index).copied()?;
        self.index += 1;
        Some(entry)
    }
}

impl PartialEq for SourceRegistryIdentityV2 {
    fn eq(&self, other: &Self) -> bool {
        self.registrations_v2().eq(other.registrations_v2())
    }
}

impl Eq for SourceRegistryIdentityV2 {}

impl core::hash::Hash for SourceRegistryIdentityV2 {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.registrations_v2().count().hash(state);
        for entry in self.registrations_v2() {
            entry.hash(state);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedSourceRegistryViewV2 {
    combined: SourceRegistryViewV2,
    parent_identity: SourceRegistryIdentityV2,
}

impl SourceRegistryViewV2 {
    pub fn try_new(
        mut entries: Vec<SourceRegistrationV2>,
    ) -> Result<Self, SourceRegistryDiagnosticV2> {
        if entries.is_empty() {
            return Ok(Self {
                identity: SourceRegistryIdentityV2(SourceRegistrationsV2::Empty),
            });
        }
        if let [entry] = entries.as_slice() {
            return Ok(Self {
                identity: SourceRegistryIdentityV2(SourceRegistrationsV2::Single(*entry)),
            });
        }
        // Check duplicates by authored ordinal before changing the input order.
        // The temporary index buffer preserves the exact first-duplicate
        // diagnostic without retaining a second registration payload.
        let mut ordinals = entries.iter().copied().enumerate().collect::<Vec<_>>();
        ordinals.sort_unstable_by_key(|(index, entry)| (entry.source_id, *index));
        let first_duplicate = ordinals
            .windows(2)
            .filter_map(|pair| match pair {
                [(_, original), (index, duplicate)]
                    if original.source_id == duplicate.source_id =>
                {
                    Some((*index, *original, *duplicate))
                }
                _ => None,
            })
            .min_by_key(|(index, _, _)| *index);
        if let Some((_, original, duplicate)) = first_duplicate {
            return Err(SourceRegistryDiagnosticV2::DuplicateSourceId {
                source_id: duplicate.source_id,
                original_len: original.source_len,
                duplicate_len: duplicate.source_len,
            });
        }
        drop(ordinals);
        // Valid registry identity is independent of host registration order.
        entries.sort_unstable_by_key(|entry| entry.source_id);
        Ok(Self {
            identity: SourceRegistryIdentityV2(SourceRegistrationsV2::Many(std::sync::Arc::new(
                entries,
            ))),
        })
    }

    pub fn try_check_location(
        &self,
        location: LocatedSpanV2,
    ) -> Result<CheckedLocatedSpanV2, SourceRegistryDiagnosticV2> {
        let source_len = self.source_len(location.source_id).ok_or(
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
        if let Some(original_len) = self.source_len(entry.source_id) {
            return Err(SourceRegistryDiagnosticV2::DuplicateSourceId {
                source_id: entry.source_id,
                original_len,
                duplicate_len: entry.source_len,
            });
        }
        let combined = match &self.identity.0 {
            SourceRegistrationsV2::Empty => SourceRegistrationsV2::Single(entry),
            SourceRegistrationsV2::Single(parent) => {
                let pair = if parent.source_id < entry.source_id {
                    [*parent, entry]
                } else {
                    [entry, *parent]
                };
                SourceRegistrationsV2::Pair(pair)
            }
            SourceRegistrationsV2::Many(parent) => SourceRegistrationsV2::ExtendedMany {
                parent: parent.clone(),
                document: entry,
            },
            // Chained public extensions retain their prior behavior. The closed
            // frontend extends a host registry once; this fallback is not an
            // allocation-qualified path.
            SourceRegistrationsV2::Pair(_) | SourceRegistrationsV2::ExtendedMany { .. } => {
                let mut entries = self.identity.registrations_v2().collect::<Vec<_>>();
                let insertion = match entries
                    .binary_search_by_key(&entry.source_id, |registered| registered.source_id)
                {
                    Ok(index) => {
                        return Err(SourceRegistryDiagnosticV2::DuplicateSourceId {
                            source_id: entry.source_id,
                            original_len: entries
                                .get(index)
                                .map_or(entry.source_len, |found| found.source_len),
                            duplicate_len: entry.source_len,
                        })
                    }
                    Err(index) => index,
                };
                entries.insert(insertion, entry);
                SourceRegistrationsV2::Many(std::sync::Arc::new(entries))
            }
        };
        Ok(ExtendedSourceRegistryViewV2 {
            combined: Self {
                identity: SourceRegistryIdentityV2(combined),
            },
            parent_identity: self.identity(),
        })
    }

    #[must_use]
    pub fn identity(&self) -> SourceRegistryIdentityV2 {
        self.identity.clone()
    }

    #[must_use]
    pub fn source_len(&self, source_id: SourceIdV2) -> Option<usize> {
        self.identity.source_len_v2(source_id)
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
#[expect(clippy::unwrap_used, clippy::panic, reason = "owner assertions")]
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
        let cloned = a.clone();
        let (SourceRegistrationsV2::Many(left), SourceRegistrationsV2::Many(right)) =
            (&a.identity.0, &cloned.identity.0)
        else {
            panic!("two-source registry must share its owned backing");
        };
        assert!(std::sync::Arc::ptr_eq(left, right));
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
        assert_eq!(extended.source_len(one.source_id), Some(one.source_len));
        assert_eq!(extended.source_len(two.source_id), Some(two.source_len));
    }

    #[test]
    fn one_document_extension_shares_host_entries_and_has_canonical_identity() {
        use core::hash::{Hash, Hasher};

        let host_entries = [
            SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(2),
                source_len: 7,
            },
            SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 15,
            },
        ];
        let host = SourceRegistryViewV2::try_new(host_entries.to_vec()).unwrap();
        for document_id in [1, 5, 10] {
            let document = SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(document_id),
                source_len: 21,
            };
            let extended = host.try_extend_document(document).unwrap();
            let SourceRegistrationsV2::Many(host_owner) = &host.identity.0 else {
                panic!("two host registrations must share one owner");
            };
            let SourceRegistrationsV2::ExtendedMany { parent, .. } = &extended.combined.identity.0
            else {
                panic!("document extension must borrow host owner");
            };
            assert!(std::sync::Arc::ptr_eq(host_owner, parent));

            let materialized = SourceRegistryViewV2::try_new(
                [host_entries[0], host_entries[1], document].to_vec(),
            )
            .unwrap();
            assert_eq!(extended.identity(), materialized.identity());
            let fingerprint = |identity: &SourceRegistryIdentityV2| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                identity.hash(&mut hasher);
                hasher.finish()
            };
            assert_eq!(
                fingerprint(&extended.identity()),
                fingerprint(&materialized.identity())
            );
            for entry in [host_entries[0], host_entries[1], document] {
                assert_eq!(extended.source_len(entry.source_id), Some(entry.source_len));
            }
            assert_eq!(
                extended
                    .try_check_location(LocatedSpanV2 {
                        source_id: document.source_id,
                        span: SpanV1::new(0, document.source_len),
                    })
                    .unwrap()
                    .registry_identity(),
                &materialized.identity()
            );
            assert_eq!(
                host.try_extend_document(host_entries[0]).unwrap_err(),
                SourceRegistryDiagnosticV2::DuplicateSourceId {
                    source_id: host_entries[0].source_id,
                    original_len: host_entries[0].source_len,
                    duplicate_len: host_entries[0].source_len,
                }
            );
        }
    }

    #[test]
    fn one_source_host_extends_inline_and_matches_materialized_pair() {
        let host_entry = SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 15,
        };
        let document = SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(2),
            source_len: 21,
        };
        let host = SourceRegistryViewV2::try_new(vec![host_entry]).unwrap();
        let extended = host.try_extend_document(document).unwrap();
        assert!(matches!(
            &extended.combined.identity.0,
            SourceRegistrationsV2::Pair([first, second]) if *first == document && *second == host_entry
        ));
        assert_eq!(
            extended.identity(),
            SourceRegistryViewV2::try_new(vec![host_entry, document])
                .unwrap()
                .identity()
        );
        let chained = extended
            .combined()
            .try_extend_document(SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(5),
                source_len: 11,
            })
            .unwrap();
        assert_eq!(
            chained.source_len(document.source_id),
            Some(document.source_len)
        );
        assert_eq!(
            chained.source_len(host_entry.source_id),
            Some(host_entry.source_len)
        );
        assert_eq!(chained.source_len(SourceIdV2::from_raw(5)), Some(11));
    }

    #[test]
    fn duplicate_diagnostic_keeps_authored_length_order() {
        let source_id = SourceIdV2::from_raw(7);
        let error = SourceRegistryViewV2::try_new(vec![
            SourceRegistrationV2 {
                source_id,
                source_len: 29,
            },
            SourceRegistrationV2 {
                source_id,
                source_len: 13,
            },
        ])
        .unwrap_err();
        assert_eq!(
            error,
            SourceRegistryDiagnosticV2::DuplicateSourceId {
                source_id,
                original_len: 29,
                duplicate_len: 13,
            }
        );

        let other = SourceIdV2::from_raw(2);
        let interleaved = SourceRegistryViewV2::try_new(vec![
            SourceRegistrationV2 {
                source_id,
                source_len: 29,
            },
            SourceRegistrationV2 {
                source_id: other,
                source_len: 41,
            },
            SourceRegistrationV2 {
                source_id,
                source_len: 13,
            },
            SourceRegistrationV2 {
                source_id: other,
                source_len: 7,
            },
        ])
        .unwrap_err();
        assert_eq!(
            interleaved,
            SourceRegistryDiagnosticV2::DuplicateSourceId {
                source_id,
                original_len: 29,
                duplicate_len: 13,
            }
        );
    }

    #[test]
    fn empty_host_to_one_source_extension_stays_inline_and_rejects_foreign_id() {
        let host = SourceRegistryViewV2::try_new(Vec::new()).unwrap();
        assert!(matches!(&host.identity.0, SourceRegistrationsV2::Empty));
        let entry = SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(11),
            source_len: 9,
        };
        let extended = host.try_extend_document(entry).unwrap();
        assert!(matches!(
            &extended.combined.identity.0,
            SourceRegistrationsV2::Single(observed) if *observed == entry
        ));
        assert_eq!(extended.source_len(entry.source_id), Some(9));
        assert_eq!(extended.source_len(SourceIdV2::from_raw(12)), None);
        assert_eq!(extended.parent_identity(), &host.identity());
        assert_eq!(
            extended.combined.identity(),
            SourceRegistryViewV2::try_new(vec![entry])
                .unwrap()
                .identity()
        );
        assert_eq!(
            extended
                .combined
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(12),
                    span: SpanV1::point(0),
                })
                .unwrap_err()
                .code(),
            "hir.source_registry.unknown_source_id"
        );
    }
    #[test]
    fn identity_is_structural_and_checked_locations_retain_identity() {
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
        assert_eq!(left.registry_identity, right.registry_identity);
    }
}
