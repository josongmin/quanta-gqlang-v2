# 15 Non-Relational Artifact Contract And Servable Boundaries

> Historical 2026-06-24 snapshot. “Current Main” below refers to that dated
> source, not today's repository. Use the [current language authority](../../../language/README.md).

Status: `Implemented On Current Main`

## Current Main Receipt

As of `2026-06-24`, explicit artifact/no-serve closure is live:

1. `CompiledItemV1` now exposes explicit `ValueArtifact` and `RuleArtifact`
   variants
2. `NoServeReasonV1` names the real boundary:
   `value_declaration`, `rule_artifact`
3. standalone decl/rule units compile successfully with `plan() == None`
   instead of relaying placeholder algebra rejects
4. CLI and public downstream facade surface the artifact/no-serve distinction;
   conformance rails pin it as part of family truth
5. proof rails:
   `v3_normalize_item_decl_rule.rs`,
   `v3_cli_item_surfacing.rs`,
   `frontend-contract/tests/downstream_contract.rs`,
   `conformance/tests/self_validation.rs`
6. verified with:
   `cargo test -p quanta-gqlang-v2-pipeline`,
   `cargo test -p quanta-gqlang-v2-cli --test v3_cli_item_surfacing`,
   `cargo test -p quanta-gqlang-v2-conformance --test self_validation`,
   `cargo test --workspace`,
   `cargo clippy --workspace --all-targets`

Honest residual:

1. current LSP remains diagnostics-oriented and does not add a separate
   artifact-rendering surface; the artifact contract stays owned by shared
   pipeline/public types plus CLI surfacing

## Goal

Define explicit compiled artifacts and explicit no-serve boundaries for
standalone non-relational units such as `let` declarations and horn rules.

## Why

The pre-closeout gap routed standalone non-relational units into placeholder
relational-algebra rejects. That is acceptable as a temporary guard, but it is
not an acceptable final contract for a SOTA-grade frontend.

The frontend needs to distinguish:

1. structurally valid but non-servable units
2. valid compiled artifacts with no runtime plan
3. invalid units that should reject immediately

## Deliverables

1. explicit compiled-artifact variants or equivalent host-agnostic contract for:
   - scalar/value declaration units
   - horn-rule units
   - future non-relational frontend artifacts if added
2. explicit serveability / no-plan / no-serve classification on those artifacts
3. diagnostics that name the real boundary rather than relaying
   `algebra.unsupported_composition`
4. public surfacing in pipeline / CLI / LSP / `frontend-contract`
5. compatibility rules for downstream semantica handoff:
   frontend artifact yes, runtime DTO no

## Pinned Semantics

1. not every valid frontend unit is a relational query
2. a valid non-relational frontend unit may compile successfully without a plan
3. no-serve is a first-class outcome, not a disguised algebra failure
4. artifact contracts remain host-agnostic and frontend-owned
5. planning still owns plan outputs; this ticket owns only artifact/no-serve
   semantics outside relational planning

## Non-Goals

1. this ticket does not redefine relational algebra:
   ticket `04`
2. this ticket does not redefine planning semantics for plan-producing families:
   ticket `07`
3. this ticket does not define semantica runtime execution of these artifacts

## Scenarios Unlocked

1. `let total = 1 + 2` compiles into an explicit value artifact with
   `servable=false` rather than pretending to be a failed relational query
2. a horn rule can compile into a logic artifact even when no runtime serving
   contract exists yet
3. CLI/LSP can surface "compiled, not servable" distinctly from "rejected"
4. downstream semantica can evolve artifact-specific handling without frontend
   runtime leakage

## Definition Of Done

1. standalone decl/rule units no longer permanently bottom out in
   `algebra.unsupported_composition`
2. the compiled artifact boundary is explicit in public pipeline types
3. tooling can distinguish compiled-no-plan from reject
4. no-serve outcomes name the real semantic boundary

## Dependencies

1. requires `00`
2. depends on `07`, `12`, and `14`
3. closes before `09` may claim non-relational packet closure

## Likely Touch Points

1. `crates/quanta-gqlang-v2-pipeline/src/compile.rs`
2. `crates/quanta-gqlang-v2-pipeline/src/lib.rs`
3. `crates/quanta-gqlang-v2-cli/src/v3_plan.rs`
4. `crates/quanta-gqlang-v2-lsp/src/handlers/v3_diagnostics.rs`
5. `crates/quanta-gqlang-v2-frontend-contract/src/lib.rs`

## Mandatory Worker Outputs

1. one explicit non-relational compiled-artifact contract
2. one explicit compiled-versus-no-serve-versus-reject distinction
3. removal of placeholder algebra rejects as the final standalone decl/rule
   contract

## Proof Rails

1. replace placeholder decl/rule edge-case rails with explicit artifact/no-serve
   rails where appropriate
2. add CLI/LSP surfacing rails for compiled-no-plan versus reject
3. keep one negative rail proving an actually invalid non-relational unit still
   rejects fail-closed

## Reject Or Escalate

1. routing valid non-relational units through permanent algebra placeholder
   failures
2. inventing runtime-native execution payloads in frontend artifact types
3. collapsing compiled-no-serve and reject into the same user-visible outcome
