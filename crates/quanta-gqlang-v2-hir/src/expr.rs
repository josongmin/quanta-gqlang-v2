//! Generic value and boolean expression HIR nodes.
//!
//! These are the shared scalar/predicate kernel embedded by query,
//! declaration, rule, relation, and result-shape surfaces. They carry node ids
//! and spans but no resolved types: typing is a later attachment point owned by
//! ticket 03. Closed operator/builtin legality and overload resolution are NOT
//! decided here — operators are open symbolic tags, function callees are open
//! references. This crate only fixes the SHAPE.

use crate::id::HirNodeIdV1;
use crate::span::SpanV1;

/// A scalar/value expression node.
///
/// Every variant is wrapped in [`ValueExprHirV1`] which pairs the [`ValueExprKindV1`]
/// payload with a node id and a span. Sub-expressions are owned (`Box`/`Vec`)
/// so a value tree is a self-contained owned tree, while still being *embedded*
/// by reference into bool/relation/decl/rule/query nodes that share this type.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueExprHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The structural payload.
    pub kind: ValueExprKindV1,
}

impl ValueExprHirV1 {
    /// Pair a kind with its id and span.
    #[must_use]
    pub fn new(id: HirNodeIdV1, span: SpanV1, kind: ValueExprKindV1) -> Self {
        Self { id, span, kind }
    }
}

/// Structural payload of a [`ValueExprHirV1`].
#[derive(Clone, Debug, PartialEq)]
pub enum ValueExprKindV1 {
    /// A literal constant.
    Literal(LiteralV1),
    /// A bare symbol reference (e.g. a binding or parameter name).
    ///
    /// Resolution to a binding/param/source is the binder's job (ticket 02);
    /// here it is just an open name.
    Symbol(SymbolRefV1),
    /// Field/path access against a base value, e.g. `a.b.c`.
    ///
    /// Represented as a base expression plus an ordered list of path segments
    /// so nested access shares one structure. `a.b` is a base `Symbol(a)` with
    /// one segment `b`.
    Path(PathAccessV1),
    /// Prefix unary operation, e.g. `-x` or `not`-style arithmetic negation.
    Unary(UnaryExprV1),
    /// Infix binary operation, e.g. `a + b`.
    Binary(BinaryExprV1),
    /// A function/operator call with positional arguments, e.g. `f(1, "x")`.
    Call(CallExprV1),
    /// An explicitly parenthesized expression. Kept as its own node so the
    /// formatter and explain layers can preserve author grouping rather than
    /// inferring it from precedence.
    Paren(Box<ValueExprHirV1>),
    /// Record (named-field) construction, e.g. `{ id: u.id, name: u.name }`.
    Record(RecordExprV1),
    /// List construction, e.g. `[u.t1, u.t2]`.
    List(ListExprV1),
}

/// A literal constant value.
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralV1 {
    /// Integer literal, kept as raw source text so no precision is assumed
    /// before typing (ticket 03 picks the numeric type).
    Int(String),
    /// Floating/decimal literal, kept as raw source text.
    Float(String),
    /// String literal (already unescaped content).
    Str(String),
    /// Boolean literal.
    Bool(bool),
    /// Null/absent literal.
    Null,
}

/// A bare symbol reference.
#[derive(Clone, Debug, PartialEq)]
pub struct SymbolRefV1 {
    /// The referenced name as written.
    pub name: String,
}

/// One segment in a path access (the `.b` in `a.b`).
#[derive(Clone, Debug, PartialEq)]
pub struct PathSegmentV1 {
    /// Field/segment name.
    pub name: String,
    /// Span of this segment alone.
    pub span: SpanV1,
}

/// Field/path access against a base value.
#[derive(Clone, Debug, PartialEq)]
pub struct PathAccessV1 {
    /// The base expression the path is rooted at.
    pub base: Box<ValueExprHirV1>,
    /// Ordered non-empty path segments applied to `base`.
    pub segments: Vec<PathSegmentV1>,
}

/// Open symbolic unary operator tag. Legality is NOT decided here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryOpV1 {
    /// Operator symbol as written (e.g. `"-"`, `"+"`).
    pub symbol: String,
}

/// Open symbolic binary operator tag. Legality is NOT decided here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryOpV1 {
    /// Operator symbol as written (e.g. `"+"`, `"-"`, `"*"`, `"/"`).
    pub symbol: String,
}

/// Prefix unary expression.
#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExprV1 {
    /// The operator tag.
    pub op: UnaryOpV1,
    /// Operand.
    pub operand: Box<ValueExprHirV1>,
}

/// Infix binary expression.
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExprV1 {
    /// The operator tag.
    pub op: BinaryOpV1,
    /// Left operand.
    pub lhs: Box<ValueExprHirV1>,
    /// Right operand.
    pub rhs: Box<ValueExprHirV1>,
}

/// A function/operator call.
#[derive(Clone, Debug, PartialEq)]
pub struct CallExprV1 {
    /// The callee, an open reference. Overload resolution is ticket 03.
    pub callee: CalleeRefV1,
    /// Positional arguments.
    pub args: Vec<ValueExprHirV1>,
}

/// An open reference to a callable.
#[derive(Clone, Debug, PartialEq)]
pub struct CalleeRefV1 {
    /// The callee name as written.
    pub name: String,
    /// Span of the callee name.
    pub span: SpanV1,
}

/// A single named field within a record construction.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordFieldV1 {
    /// Field name.
    pub name: String,
    /// Span of the field name.
    pub name_span: SpanV1,
    /// Field value expression.
    pub value: ValueExprHirV1,
}

/// Record (named-field) construction.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordExprV1 {
    /// Ordered fields. Order is preserved; duplicate-name legality is a later
    /// ticket's concern.
    pub fields: Vec<RecordFieldV1>,
}

/// List construction.
#[derive(Clone, Debug, PartialEq)]
pub struct ListExprV1 {
    /// Ordered elements.
    pub elements: Vec<ValueExprHirV1>,
}

/// A boolean/predicate expression node.
///
/// Embeds [`ValueExprHirV1`] for comparison operands, proving the value kernel
/// is shared rather than re-declared. Quantifier-style forms exist as nodes
/// only (no binding/scoping semantics — that is ticket 02).
#[derive(Clone, Debug, PartialEq)]
pub struct BoolExprHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The structural payload.
    pub kind: BoolExprKindV1,
}

impl BoolExprHirV1 {
    /// Pair a kind with its id and span.
    #[must_use]
    pub fn new(id: HirNodeIdV1, span: SpanV1, kind: BoolExprKindV1) -> Self {
        Self { id, span, kind }
    }
}

/// Structural payload of a [`BoolExprHirV1`].
#[derive(Clone, Debug, PartialEq)]
pub enum BoolExprKindV1 {
    /// A boolean literal predicate.
    Literal(bool),
    /// A value expression used in boolean position (e.g. a bare `z`).
    Value(Box<ValueExprHirV1>),
    /// Comparison between two value expressions, e.g. `x > 1`.
    Compare(CompareExprV1),
    /// Logical conjunction `lhs and rhs`.
    And(BinaryBoolV1),
    /// Logical disjunction `lhs or rhs`.
    Or(BinaryBoolV1),
    /// Logical negation `not inner`.
    Not(Box<BoolExprHirV1>),
    /// Explicit parenthesized grouping. Preserved as a node so author grouping
    /// such as `(y = 2 or z)` is not flattened away by precedence.
    Paren(Box<BoolExprHirV1>),
    /// Quantifier-style placeholder (e.g. `exists` / `forall`). Node-only:
    /// binding and scope semantics are owned by ticket 02; here it merely holds
    /// an open quantifier tag and a body predicate.
    Quantifier(QuantifierExprV1),
}

/// Open symbolic comparison operator tag. Legality is NOT decided here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareOpV1 {
    /// Comparison symbol as written (e.g. `">"`, `"="`, `"<="`).
    pub symbol: String,
}

/// Comparison between two value expressions.
#[derive(Clone, Debug, PartialEq)]
pub struct CompareExprV1 {
    /// The comparison operator tag.
    pub op: CompareOpV1,
    /// Left operand.
    pub lhs: Box<ValueExprHirV1>,
    /// Right operand.
    pub rhs: Box<ValueExprHirV1>,
}

/// A binary boolean connective body (`and` / `or`).
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryBoolV1 {
    /// Left predicate.
    pub lhs: Box<BoolExprHirV1>,
    /// Right predicate.
    pub rhs: Box<BoolExprHirV1>,
}

/// Open symbolic quantifier tag (placeholder only).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuantifierKindV1 {
    /// Quantifier keyword as written (e.g. `"exists"`, `"forall"`).
    pub keyword: String,
}

/// Quantifier-style placeholder node.
#[derive(Clone, Debug, PartialEq)]
pub struct QuantifierExprV1 {
    /// The open quantifier tag.
    pub quantifier: QuantifierKindV1,
    /// Bound-variable names as written. No scoping is applied here.
    pub binders: Vec<String>,
    /// The quantified body predicate.
    pub body: Box<BoolExprHirV1>,
}
