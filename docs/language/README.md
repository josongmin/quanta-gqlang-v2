# QGLang Language Surface

## Current Status

`quanta-gqlang-v2` is now a `v3` frontend-only repo.

The legacy monolithic `syntax -> sema -> ir` path is gone. The current
`quanta-gqlang-v2-sema` crate is a stage owner inside the live pipeline, not a
public downstream entrypoint; the deleted legacy syntax/IR crate contracts are
not compatibility surfaces.

The live owner stack is:

1. `surface`: parser, document parser, formatter
2. `hir`: shared item / expr / relation / result-shape carriers
3. `binder`: lexical scope, capture, module-graph-aware binding
4. `types`: closed type universe, overload legality, type env
5. `modules`: pack, manifest, lockfile, catalog, import graph
6. `algebra`: normalized relational semantics
7. `logic`: recursive predicate, graph, witness, recursion legality
8. `planning`: truth, evidence, effect, result-kind, cost
9. `pipeline`: one shared compiler service
10. `frontend-contract`: downstream facade

## Public Compile Surface

`quanta-gqlang-v2-frontend-contract` is the only public downstream boundary.
It exposes two explicit authorities:

1. `ClosedFrontendAuthorityV2`, built through
   `ClosedFrontendAuthorityBuilderV2`: closed compilation with caller-supplied
   schema, catalog, module, pack, and lockfile authority
2. `OpenAnalysisAuthorityV2`: syntax-preserving analysis without fabricated
   external authority

The facade's `compile_source_document_v3` binds original source bytes and
checked authority into `CompiledSourceDocumentV3` and its immutable
`ExecutionDocumentV3`. Open analysis does not issue that document. The exact
producer contract and unsupported payloads are in
[frontend execution V3](../contracts/frontend-execution-v3.md); runtime
admission and execution stay downstream-owned.

Interpretation rules:

1. `AnalysisStatusV2::Complete` means the analysis has all required authority
   proofs; a fully populated `OpenAnalysisAuthorityV2` can produce it too
2. missing external authority under open analysis produces
   `AnalysisStatusV2::Incomplete`, never a fabricated plan; even complete open
   analysis does not issue an `ExecutionDocumentV3`
3. parse or semantic rejection produces `AnalysisStatusV2::Error`
4. CLI, diagnostics stdio, conformance, and downstream consumers import only
   the frontend contract; pipeline internals are not a public seam
5. source parsing rejects relation paths deeper than the checked
   `FrontendBudgetV2::max_relation_depth` before HIR integrity traversal;
   the HIR owner also checks programmatically assembled trees before its
   recursive integrity walk, and Sema checks the configured budget before
   semantic traversal. The standard and maximum configurable value is 128
6. source parsing rejects expression trees deeper than
   `FrontendBudgetV2::max_expression_depth`, including left-associated operator
   chains, before formatting or semantic traversal; the maximum configurable
   value is 64; programmatic HIR also bounds nested result shapes to 64
7. parser-produced HIR uses the infallible canonical formatter; caller-built
   raw HIR should use `try_format_item` for surface representability. Its
   stack-safe full-tree comparison ignores ids, spans, and semantic-neutral
   parentheses but rejects every other parse/format structural drift. It does
   not replace HIR integrity checks or downstream semantic validation
8. direct Logic callers can tune `max_rule_program_rules` independently of
   graph nodes, edges, and steps before sealing authority; raw programs over
   the rule or literal cap reject before graph construction
9. local module lookup resolves declarations before direct imports before
   re-exports. A direct import is never an implicit downstream export:
   downstream lookup resolves the target module's declaration first (enforcing
   its visibility), then explicit re-export edges, so a direct import cannot
   suppress downstream re-export ambiguity

## Surface Families

Current public breadth covered by the shared pipeline:

1. relational query
2. graph query / `find` subkinds:
   `reachability`, `flow`, `slice`, `taint`, `path`, `points_to`, `summary`
3. `check`
4. presentation families:
   `explain`, `show`, `describe`, `sample`, `compare`, `export`
5. catalog registration families:
   `view`, `model`, `pack`, `profile`, `gate`, `ruledef`
6. meta families:
   `use pack`, `require certified`
7. composition:
   `let ... with ...`
8. supplemental non-relational units:
   standalone `let` -> `ValueArtifact`
   standalone horn `rule` -> `RuleArtifact`

Current non-servable truth:

1. standalone `let` compiles to `ValueArtifact` with `NoServeReasonV1::ValueDeclaration`
2. standalone horn `rule` compiles to `RuleArtifact` with
   `NoServeReasonV1::RuleArtifact`
3. unsupported mixes reject fail-closed with typed planning or logic codes
4. nothing is down-cast into legacy `RootSpecV1`

## CLI And Diagnostics Stdio

CLI:

1. `query` and `v3-query` are the same command
2. `fmt` and `v3-fmt` are the same command
3. `query` parses multi-item documents through open analysis, without a host
   catalog or execution; unresolved external authority reports `incomplete`
   with exit code 1
4. rejected analysis emits canonical diagnostics and exits non-zero; a
   successful `fmt` command is not evidence of closed compilation

Diagnostics stdio:

1. the read-only analysis method set is:

<!-- supported-diagnostics-methods:start -->
- `v3_diagnostics`
<!-- supported-diagnostics-methods:end -->

2. it runs through `OpenAnalysisAuthorityV2`
3. it returns canonical status plus ordered typed diagnostics
4. no legacy `hover` / `completion` / `symbols` / `diagnostics` / `zero_query`
   method remains
5. it is bounded NDJSON with exact request fields, not JSON-RPC or LSP; it does
   not implement `initialize` or document lifecycle methods

## Examples

The [examples index](examples_index.md) is the single maintained list of
repo-owned example paths. It separates query, presentation, declaration, meta,
and composition examples without claiming runtime execution.

## Proof Rails

The [test hellgate ledger](test_hellgate_ledger.md) maps current frontend
contracts to their primary owner, process, conformance, and policy rails. Cargo
and nextest discover the Rust test inventory; this README does not keep
a second file list.

## Boundary

This repo owns:

1. syntax
2. sema/binder/types/modules
3. HIR
4. normalized frontend semantics
5. planning IR
6. CLI/diagnostics-stdio/formatter
7. frontend-facing downstream contract

This repo does not own:

1. runtime substrate
2. provider registry routing
3. backend DTOs
4. semantica-native serving request contracts
5. execution/materialization claims
