//! Exhaustive semantics rails for [`SpanV1`].
//!
//! `SpanV1` is the source-mapping primitive every HIR node carries, so each of
//! its constructors and predicates must behave exactly as documented: `checked`
//! is the fail-closed constructor (rejects inverted ranges), `len`/`is_empty`
//! agree at the zero-width boundary, `is_within`/`is_well_formed` are the
//! integrity-gate predicates, and `merge` returns the tight hull. These rails
//! lock the real arithmetic, not a smoke test.

use quanta_gqlang_v2_hir::SpanV1;

#[test]
fn new_stores_offsets_verbatim_without_validation() {
    // `new` is explicitly unchecked: it must store an inverted range as-is so the
    // integrity gate (not the constructor) is the one rejecting it.
    let inverted = SpanV1::new(40, 10);
    assert_eq!(inverted.start, 40);
    assert_eq!(inverted.end, 10);
    assert!(!inverted.is_well_formed());
}

#[test]
fn checked_accepts_well_formed_and_zero_width() {
    assert_eq!(SpanV1::checked(3, 9), Some(SpanV1::new(3, 9)));
    // start == end is the boundary case and must be accepted (zero-width).
    assert_eq!(SpanV1::checked(7, 7), Some(SpanV1::new(7, 7)));
}

#[test]
fn checked_rejects_inverted_range() {
    // The fail-closed reject: start > end yields None, never a bogus span.
    assert_eq!(SpanV1::checked(9, 3), None);
}

#[test]
fn point_is_zero_width_at_offset() {
    let p = SpanV1::point(12);
    assert_eq!(p.start, 12);
    assert_eq!(p.end, 12);
    assert_eq!(p.len(), 0);
    assert!(p.is_empty());
    assert!(p.is_well_formed());
}

#[test]
fn len_is_byte_width_and_saturates_on_inverted() {
    assert_eq!(SpanV1::new(4, 10).len(), 6);
    // saturating_sub means an inverted span reports 0, not a wraparound.
    assert_eq!(SpanV1::new(10, 4).len(), 0);
}

#[test]
fn is_empty_tracks_zero_or_negative_width() {
    assert!(SpanV1::new(5, 5).is_empty());
    assert!(SpanV1::new(10, 4).is_empty(), "inverted counts as empty");
    assert!(!SpanV1::new(5, 6).is_empty());
}

#[test]
fn is_well_formed_is_start_le_end() {
    assert!(SpanV1::new(0, 0).is_well_formed());
    assert!(SpanV1::new(0, 1).is_well_formed());
    assert!(!SpanV1::new(1, 0).is_well_formed());
}

#[test]
fn is_within_requires_well_formed_and_in_bounds() {
    // end == source_len is the inclusive upper boundary and must pass.
    assert!(SpanV1::new(0, 10).is_within(10));
    assert!(SpanV1::new(2, 5).is_within(10));
    // end past source_len fails closed.
    assert!(!SpanV1::new(0, 11).is_within(10));
    // an inverted span is never "within" even if numerically small.
    assert!(!SpanV1::new(8, 3).is_within(10));
}

#[test]
fn merge_is_the_tight_covering_hull() {
    let a = SpanV1::new(2, 5);
    let b = SpanV1::new(8, 12);
    // disjoint: hull spans min(start)..max(end).
    assert_eq!(a.merge(&b), SpanV1::new(2, 12));
    // commutative.
    assert_eq!(b.merge(&a), SpanV1::new(2, 12));
    // overlapping: still the outer hull.
    assert_eq!(
        SpanV1::new(1, 6).merge(&SpanV1::new(4, 9)),
        SpanV1::new(1, 9)
    );
    // nested: the larger span absorbs the smaller.
    assert_eq!(
        SpanV1::new(0, 20).merge(&SpanV1::new(5, 8)),
        SpanV1::new(0, 20)
    );
}

#[test]
fn default_span_is_zero_point() {
    let d = SpanV1::default();
    assert_eq!(d, SpanV1::new(0, 0));
    assert!(d.is_empty());
}
