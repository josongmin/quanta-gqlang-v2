# Receipt 02 — Public-API Inventory Proposal (Lane W1-A / SS-02G)

**No implementation delta is claimed by this lane.** Product source
(`crates/**`) was read-only for this lane; the only file created is this
artifact. Final Receipt 02 (`receipts/02.md`) is integrator-reserved.

## Assignment Manifest

```text
work_package=SS-02G — Surface/HIR public-API inventory, owned-path hashes, untracked-test enrollment spec
base_sha=ecc38f59e9fdcd62b5e030445188ffa8b01e100a (dispatch freeze)
overlay_status_hash=999a0db0875be0ea... (dispatch-declared)
live_refreeze_at_lane_start=HEAD e9dd3e4c4db656cd8109478bcc1ccfd923ff7baf ("w0: freeze base sha + overlay status hash..."), 177 short-status entries, status-list sha256 2e0bf1fc13f3c285e93e77eb235ac7e3dcc2d94fe714216982969d24a4175747
allowed_write_paths=docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/02-public-api-inventory.md (this file only)
integrator_reserved_paths=receipts/02.md; all packet docs/manifests/indexes; crates/** (read-only for this lane); EXECUTION_LEDGER.md; root/Crate Cargo files
frozen_input_paths_and_hashes=see "Owned-Path Hash Manifest" below (23 files, sha256 first-16)
expected_red=none — read-only inventory lane; no product mutation, no RED command applies
acceptance=single four-argument relation-source constructor path proven with file:line evidence; overload/legacy-alias absence proven; untracked name-span test located with exact enrollment action; exact owned-path hash manifest; real public-API tool attempted with verbatim result or BLOCKED_TOOLING
receipt_path=docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/02-public-api-inventory.md
deferred=git enrollment of v3_relation_source_name_span.rs (integrator); cargo semver-checks blocked on rustc 1.81; two-sided cargo-public-api diff vs base_sha needs a baseline worktree forbidden to this lane
```

Note: live HEAD differs from the dispatch base_sha because the integrator
committed the W0 freeze after declaring it. Per wave-plan re-freeze rules the
live observation is recorded above; all inventory below was taken on this
working tree.

## 1. The Single `relation_source(name, name_span, alias, span)` Constructor Path

### Definition (HIR)

- `crates/quanta-gqlang-v2-hir/src/build.rs:300-315` — the one and only
  constructor:

  ```rust
  pub fn relation_source(
      &self,
      name: impl Into<String>,
      name_span: SpanV1,
      alias: Option<AliasV1>,
      span: SpanV1,
  ) -> RelationExprHirV1
  ```

  It mints the node ID via the builder (`self.relation(...)`) and constructs
  `RelationExprKindV1::Source(RelationSourceV1 { name, name_span, alias })`
  at `build.rs:309-313`. `name_span` is an explicit required argument; there
  is no default, no inference, and no alias-inclusive fallback.

- `crates/quanta-gqlang-v2-hir/src/relation.rs:54-61` — the carrier
  `pub struct RelationSourceV1 { pub name: String; pub name_span: SpanV1; pub alias: Option<AliasV1> }`.
  Field doc at `relation.rs:57`: "Exact source byte range of the name,
  excluding any following alias."

- Integrity traversal: `crates/quanta-gqlang-v2-hir/src/visit.rs:194-198`
  routes `source.name_span` (and the alias span) through
  `visit_auxiliary_span`; `crates/quanta-gqlang-v2-hir/src/integrity.rs:409-427`
  (`record_auxiliary_span`) rejects inverted (`InvertedAuxiliarySpan`,
  code `hir.integrity.inverted_auxiliary_span`, integrity.rs:136/414) and
  out-of-bounds (`AuxiliarySpanOutOfBounds`,
  `hir.integrity.auxiliary_span_out_of_bounds`, integrity.rs:137/416) name
  spans.

### Surface ingress (the only production caller)

- `crates/quanta-gqlang-v2-surface/src/query.rs:72-80` — private
  `fn parse_source` captures the exact lexer token span via
  `cur.expect_ident("a relation source name")` (query.rs:73), computes the
  alias-inclusive relation `span` separately (query.rs:75-78), and calls the
  four-argument builder at **`query.rs:79`**:
  `cur.builder.relation_source(name, name_span, alias, span)`.
- `parse_source` is reached from `parse_query` (query.rs:49), `parse_join`
  (query.rs:187), and `parse_cross_join` (query.rs:199) — all inside
  `surface/src/query.rs`; every public parse entry point
  (`parse_document`, `parse_program`, `parse_source_document_v2` at
  `surface/src/parser.rs:228`) routes through these, so there is no second
  parser-side ingress.

### Caller census (160 `.relation_source(` occurrences under `crates/`)

| Class | File | Call sites |
|---|---|---:|
| **Production** | `surface/src/query.rs` | 1 (line 79) |
| Feature-gated test support (`#[cfg(feature = "test")] #[doc(hidden)]`, sema `lib.rs:19-21`) | `sema/src/test_support.rs` | 4 (lines 78, 81, 87, 89) |
| HIR owner tests | `hir/tests/v3_relation_source_name_span.rs` (2, untracked), `v3_item_identity.rs` (5), `v3_item_hellgate.rs` (3), `v3_visit_traversal.rs` (4), `v3_hir_expr_kernel.rs` (3) | 17 |
| Binder tests | `v3_binder_scope.rs` (18), `v3_relation_source_use.rs` (11), `v3_binder_hellgate.rs` (4), `v3_binder_mutants.rs` (3), `v3_catalog_inline_view.rs` (2), `v3_composition_hellgate.rs` (1) | 39 |
| Types tests | `v3_types_check_edges.rs` (6), `v3_types_hellgate.rs` (2), `v3_rule_type_solver.rs` (1) | 9 |
| Sema tests | `v3_document_transaction.rs` (45), `v3_composition_query.rs` (12) | 57 |
| Logic tests | `logic/tests/v3_logic_source.rs` (line 29) | 1 |
| Non-compiling historical markdown | `algebra/tests/RETIRED_LAWS.md` | 32 (text only) |

Total: 128 compiling call sites (1 production + 4 feature-gated test-support +
123 test files) + 32 markdown mentions. Every observed call site passes four
arguments (name, name_span, alias, span); no three-argument or defaulted call
exists.

### Proof of no compatibility overloads or legacy alias constructors

1. `rg -n "fn relation_source" crates` returns exactly two hits:
   the constructor definition `hir/src/build.rs:300` and an unrelated test
   function named `relation_source_name_span_is_exact_after_multibyte_prefix`
   (`surface/tests/v3_document_hellgate.rs:123`). There is exactly one
   constructor definition workspace-wide.
2. `rg -n "impl RelationSourceV1" crates` returns **zero** hits: the type has
   no inherent constructors (`new`, `with_span`, `legacy`, alias-inclusive
   helpers) — only `#[derive(Clone, Debug, PartialEq)]`
   (relation.rs:53) and std blanket impls.
3. The `cargo public-api` listing of `quanta-gqlang-v2-hir` (see §4) contains
   exactly **one** `HirBuilderV1::relation_source` item — the four-argument
   signature — and `RelationSourceV1`'s API surface is the three pub fields
   plus derived/blanket trait impls only.
4. `rg -ni "deprecated|legacy|compat"` over both owner `src/` trees surfaces
   only the v2-surface *parser* compat policy (surface `parser.rs:8-18`,
   `lib.rs:7-16` — v3-only parsing, legacy stack removed) and doc-comments on
   unrelated carriers (`item.rs:41`, `parser.rs:147`); none is a
   relation-source constructor or span fallback.
5. `RelationExprKindV1::Source(...)` construction sites in compiling
   non-test code: only `hir/src/build.rs:309` (inside the constructor). All
   other `Source(` hits are consumer `match` arms (surface `format.rs:88,153`;
   binder `resolve.rs:690`; types `check.rs:1109,1172,1381`; sema
   `document.rs:924,1535,3045`; pipeline `compile.rs:1086`) or test-side
   struct literals (`hir/tests/v3_item_hellgate.rs:433`,
   `planning/tests/v3_item_hellgate.rs:299`) that bypass ID minting and are
   covered by the same integrity bounds checks.

Caveat recorded honestly: `RelationSourceV1`'s fields are `pub`, so external
tests can struct-literal-construct it without the builder (as the two test
literals above do). This does not change the constructor-path claim: the one
public constructor *function* is the four-argument builder method, it is the
only ID-minting path, and no overload or fallback exists. Narrowing the pub
fields would be a Ticket-02 owner decision, not this lane's to make.

## 2. Untracked Name-Span Test — Current Wiring And Intended Enrollment

- File: `crates/quanta-gqlang-v2-hir/tests/v3_relation_source_name_span.rs`
  (34 lines, 2 tests: `source_name_auxiliary_span_out_of_bounds_rejects_even_if_relation_is_in_bounds`
  at line 8 and `source_name_auxiliary_span_inverted_rejects` at line 27).
- Current git state: **untracked** (`??` in `git status --short`).
- Current build wiring: **already enrolled at the Cargo level**.
  `crates/quanta-gqlang-v2-hir/Cargo.toml` declares only `[lib]` — there is
  no `[[test]]` allowlist and no `autotests = false` — so Cargo auto-discovers
  every `tests/*.rs` as a test target. Verified live with
  `CARGO_TARGET_DIR=/tmp/qg-wave-w1a-inventory cargo test -p quanta-gqlang-v2-hir --test v3_relation_source_name_span`
  → `2 passed; 0 failed`, exit 0. The file already executes inside the
  72/72 HIR owner suite.
- The enrollment gap is therefore purely VCS tracking: the test is not part
  of any committed/tracked inventory, so owner-suite evidence is not bound to
  a frozen fingerprint (exactly what Ticket 02's "Exact remaining work" item 1
  and the SS-02F re-audit flag).
- **Exact enrollment action (NOT performed — git state-mutating, integrator
  reserved; no manifest change is needed):**

  ```bash
  git add crates/quanta-gqlang-v2-hir/tests/v3_relation_source_name_span.rs
  git status --short -- crates/quanta-gqlang-v2-hir/tests/v3_relation_source_name_span.rs   # expect "A "
  ```

  then include the file in the Receipt 02 tracked-file inventory and hash
  table. No `Cargo.toml` or `[[test]]` entry is required; auto-discovery is
  the intended wiring.
- Adjacent observation for the integrator's tracked-file inventory: two
  further untracked surface test files exist —
  `crates/quanta-gqlang-v2-surface/tests/v3_doc_inventory.rs` and
  `crates/quanta-gqlang-v2-surface/tests/v3_examples_corpus.rs`. They are
  outside this lane's item but belong in the same tracked-inventory decision.

## 3. Owned-Path Hash Manifest (sha256 first-16, `shasum -a 256`)

All files under `crates/quanta-gqlang-v2-surface/src/` and
`crates/quanta-gqlang-v2-hir/src/`, recursively — 23 files, none omitted:

```text
da8c86eaa2e0183d  crates/quanta-gqlang-v2-hir/src/build.rs
473dbe040cb2ab44  crates/quanta-gqlang-v2-hir/src/expr.rs
0e0950df1ba6ea81  crates/quanta-gqlang-v2-hir/src/id.rs
cc4858369c0109b3  crates/quanta-gqlang-v2-hir/src/integrity.rs
68c3863053f9ba13  crates/quanta-gqlang-v2-hir/src/item.rs
a75f4a45d9771afd  crates/quanta-gqlang-v2-hir/src/lib.rs
76008c968129495d  crates/quanta-gqlang-v2-hir/src/relation.rs
dc3b20c48eb6386f  crates/quanta-gqlang-v2-hir/src/shape.rs
9e0edfb2f8226d62  crates/quanta-gqlang-v2-hir/src/span.rs
8d3d1a560b284002  crates/quanta-gqlang-v2-hir/src/unit.rs
ee0a6ec341459e2b  crates/quanta-gqlang-v2-hir/src/visit.rs
4f8f4269ebb700e5  crates/quanta-gqlang-v2-surface/src/budget.rs
18a486b84d58f3ec  crates/quanta-gqlang-v2-surface/src/cursor.rs
ec89674934db0d2a  crates/quanta-gqlang-v2-surface/src/diag.rs
4b9df974a9cf8f3d  crates/quanta-gqlang-v2-surface/src/expr.rs
bb14da1f759fca66  crates/quanta-gqlang-v2-surface/src/family.rs
f78f7bd621c8afe4  crates/quanta-gqlang-v2-surface/src/format.rs
d35fc1d0e2b3db55  crates/quanta-gqlang-v2-surface/src/lexer.rs
570d6db19b3541dd  crates/quanta-gqlang-v2-surface/src/lib.rs
a85935d615075afc  crates/quanta-gqlang-v2-surface/src/parser.rs
ede0c5459adba826  crates/quanta-gqlang-v2-surface/src/query.rs
02c6a5279b7f848b  crates/quanta-gqlang-v2-surface/src/shape.rs
c22c9d90438ec2dc  crates/quanta-gqlang-v2-surface/src/unit.rs
```

Continuity note: `build.rs`, `relation.rs`, `visit.rs`, and `query.rs` match
the full hashes recorded in the historical `receipts/02.md` delta block
(da8c86ea…, 76008c96…, ee0a6ec3…, ede0c545…), i.e. these four producer files
are byte-identical to that checkpoint; the other 19 files are additional
frozen inputs this manifest adds.

## 4. Public-API Tool Results

### `cargo public-api` — AVAILABLE, ran successfully on both crates

Commands and results (lane-local capture; reproduce with the same commands):

```text
$ cargo public-api -p quanta-gqlang-v2-hir
exit=0   # 4621-item public API listing
$ cargo public-api -p quanta-gqlang-v2-surface
exit=0   # 545-item public API listing
```

Decisive excerpts (grep counts over the full listings):

- `grep -c "HirBuilderV1::relation_source"` over the HIR listing = **1**:

  ```text
  pub fn quanta_gqlang_v2_hir::HirBuilderV1::relation_source(&self, name: impl core::convert::Into<alloc::string::String>, name_span: quanta_gqlang_v2_hir::SpanV1, alias: core::option::Option<quanta_gqlang_v2_hir::AliasV1>, span: quanta_gqlang_v2_hir::SpanV1) -> quanta_gqlang_v2_hir::RelationExprHirV1
  ```

  This is the tool-level proof that the public API exposes exactly one
  relation-source constructor and it is the four-argument form with
  `name_span` explicit.

- `grep -c "relation_source"` over the Surface listing = **0**: Surface
  exposes no relation-source constructor; its public surface is the
  parse/format entry points (e.g. `parse_source_document_v2`).

- `RelationSourceV1` in the HIR listing: `pub struct` + three pub fields
  (`name`, `name_span`, `alias`) + derived `Clone`/`PartialEq`/`Debug` and
  std blanket impls only. No user-defined inherent constructor appears in the
  public API.

Tool-diff scope note: a two-sided `cargo public-api diff` against the frozen
`base_sha` needs a baseline rustdoc build of that tree (published-version or
`--baseline rustdoc.json`); the packet crates are unpublished, and building
the baseline requires a `git checkout`/worktree — git state mutation is
forbidden to this lane and worktrees are disallowed until the user-authorized
checkpoint commit. The full-tree listings above are complete and reproducible;
the baseline diff is deferred to the integrator.

### BLOCKED_TOOLING: `cargo semver-checks`

```text
$ cargo semver-checks
error: rustc version is not high enough: >=1.93.0 needed, got 1.81.0

HELP: to use the latest rustc, run `rustup update stable && cargo +stable semver-checks <args>
exit=101
```

No grep-based pseudo-diff was substituted. The manual inventory in §1–§3
stands on its own evidence (rg census + `cargo public-api` machine listing).

## 5. Non-Claims

- No implementation delta is claimed by this lane.
- No git state was mutated; no file outside
  `receipts/02-public-api-inventory.md` was written.
- No workspace, hosted-CI, clean-candidate, or Receipt-02-closure claim.
- Owner-suite reruns (HIR 72/72, Surface 212/212) are the integrator's W1-Q
  qualification step on one frozen fingerprint; this lane only verified the
  untracked test compiles and passes in isolation (2/2).
