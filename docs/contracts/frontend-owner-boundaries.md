# Frontend owner inputs and stage evidence

The supported downstream seam is `quanta-gqlang-v2-frontend-contract`. Individual
owner crates expose inputs needed by the in-workspace stage chain; a public owner
carrier is not automatically closed compilation authority.

## Budget authority

Sema's `FrontendBudgetV2` owns the complete frontend profile. Surface owns the
closed `ParseBudgetDimensionV2` vocabulary and positivity/depth validation.
Pipeline projects each parse dimension by an exhaustive enum match through
`ParseBudgetV2::try_from_dimensions`; it supplies no independent default profile.
The parser consumes tokens, item count, expression nodes/depth, relation depth,
collection width and projection width. Pipeline separately enforces source bytes;
transport frame, semantic traversal, module/rule/authority and response limits
remain with their respective owners, not implicitly with the parser.

A new parse dimension requires updating the Surface budget and its enum, and
Pipeline must explicitly map the new variant. Adding a non-parse frontend cap
alone does not imply that parsing must consume it. Distinct-value regression
oracles pin the existing cross-owner mapping without deriving expected values
from the getters under test.

Module/relation/catalog admission uses the one Sema `authority_profile`
projection. Each admission has its own bounded counter. Later stages consume the
immutable `downstream_receipt` retained by the sealed document. These projections
are stage contracts, not independently chosen default budgets.

Conversions of positive u32 caps use `usize::try_from(...).unwrap_or(usize::MAX)`.
On supported 32/64-bit targets the conversion is exact. Even on a narrower target,
that fallback reduces an unrepresentable cap to the largest representable value;
it cannot increase the owner's cap. It is not an unlimited-budget fallback.
Depth constraints still apply before recursive traversal.

## Binder input boundary

`Binder::resolve_*` consumes structurally checked, depth-bounded HIR. It does not
admit arbitrary untrusted trees itself. The source transaction passes parser
budget checks, checked HIR integrity and Sema's configured preflight before
calling Binder. Direct owner users must uphold the same checked-HIR prerequisite.

Production Sema gives Binder a read-only `ModuleImportAdapterV2` over the Modules
owner's sealed `ResolvedGraphV2`. Binder's `ModuleGraphV1` is an inert snapshot
used for isolated inputs/tests; its `with_import` helper does not mint a Modules
proof, closed authority or semantic document. Ambiguous targets stay ambiguous.

## Semantic state boundary

`DeclaredDocumentV2` records checked syntax and declared identity membership;
body resolution and catalog validation still have to run. The former
`ResolvedDocumentV2::from_declared` only wrapped this input, before resolution
actually ran, and supplied no additional evidence. That wrapper and its owner
re-export are removed. The transaction now passes declared input directly to
body resolution, validates the resulting catalog and calls `try_assemble`.

This removal changes the unsupported direct Sema-owner API; it does not remove a
root from the curated downstream facade. No workspace consumer other than Sema's
own transaction used the wrapper. There is no compatibility alias claiming a
resolution proof that has not been produced.

`IncompleteSemanticDocumentV2` is a separate editor result: construction requires
at least one nonempty-code, same-source unresolved fact. It has no conversion to
closed semantics. Public read-only accessors do not turn it into a successful
compile result. Downstream runtime execution and overflow enforcement remain
outside this repository's frontend qualification.
