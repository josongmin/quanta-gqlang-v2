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
