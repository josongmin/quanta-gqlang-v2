# Sep-27 IR: structural remediation plan

Status: PLAN ONLY. No implementation or qualification closure is claimed.

## Source baseline and scope

- QGLang: `6caf35a1cc785fa97c860b38c02736f91930e96d`, clean before this plan was added.
- During plan closeout, concurrent changes appeared in `crates/quanta-gqlang-v2-logic/src/source.rs` and `crates/quanta-gqlang-v2-types/src/check/value.rs`. They are not owned or qualified by this plan-writing turn. Preserve them and re-freeze ownership/source before implementation or receipts.
- Semantica: `63f1c4b5d370fbdc91eeeb86eefa438ab615413d`, dirty. This is an observed planning baseline, not an integration candidate. Re-freeze both repositories at implementation handoff.
- Current source was inspected directly. Earlier audit text supplies counterexamples, not implementation authority.
- Cover the confirmed Composition source/capture defect, V3 existential execution support, runtime error classification, and source-to-process proof gaps.
- Previously repaired polarity, type fanout, flattening and transparent-parenthesis behavior remain regression constraints. Do not reopen them without a current counterexample.

## Acceptance scope

The target is executable V3 existential queries over the existing issued data sources:

1. Composition with one currently admitted SAMPLE or canonical-files producer; its relation body may scan that admitted binding repeatedly.
2. Standalone Relation over issued canonical workspace files, using the same relation compiler and evaluator as Composition bodies.
3. `Scan`, ordinary scalar `Filter`, current identity/path `Project`, `SemiJoin`, `AntiJoin`, and flat `ExistsFilter`; nested existential relations within an explicit depth/work bound.
4. Single and multiple EXISTS, mixed EXISTS/NOT EXISTS, uncorrelated and genuinely correlated predicates, and ordinary residual filters.
5. Existing public one-column RepoPath output. General output schemas, new host catalogs/providers, additional producers, Group/Aggregate/Sort/general Join, and scalar/OR-embedded EXISTS are outside this increment and must remain explicit typed rejections.

The concrete scalar capability set must be frozen before parallel writers start: literals, exact binding/path access, Bool wrappers/connectives, and checked comparison overloads needed by the admitted RepoPath queries. Unsupported overloads reject before producer acquisition; operator spelling is never dispatch authority. Optional/NULL behavior follows the existing language contract; if an Optional overload is not implemented, admission rejects it rather than inventing SQL-like behavior.

Standalone Relation needs issued source/schema/provider authority; removing the current Composition guard alone does not implement it. Arbitrary `records` is not admitted by fabricating a catalog. These limits must appear in capability claims and rejection tests.

## Architecture decisions to freeze first

### Frontend ownership

- Relation source resolution and row-expression correlation are distinct facts.
- Keep exact Composition LET source resolution and the existing scan binding proof. Do not put relation-source HIR IDs in the expression correlation vector.
- Preserve exact outer-row expression captures, nested propagation, ordering, and composition scope barriers.
- Keep Sema and Algebra expression-membership/integrity checks. Do not manufacture an expression for a scan or discard bad edges downstream.
- Existing Execution V3 payloads already carry operator, expression, polarity and correlation facts. No new frontend public payload is justified by these fixes.

### Runtime ownership

- One bounded compiler consumes sealed `ExecutionRelationV3` into an opaque runtime-owned validated plan. Composition and standalone Relation share it.
- Validate support before producer/source acquisition. Retain the validated plan with the admitted request; execution and public projection consume that same plan rather than repeatedly reclassifying it.
- Family dispatch uses `AlgebraFamilyV2`; scalar dispatch uses canonical overload identities. HIR, source-text reparsing, keyword matching and V1 logical-plan detours are not fallback execution paths.
- Source-stage keys are provenance and may repeat. Assign private operator positions from bounded tree traversal; do not use `stage` as a unique operator ID.
- Keep producer input schema and relation output schema distinct even while the supported projection is narrow.
- Error classification belongs to this compiler: unsupported family/shape, wrong scan authority, unavailable expression overload, and invalid identity remain distinct typed failures. The current Project-over-non-Scan diagnosis is absorbed here; do not add a temporary second classifier.

### Existential semantics and reuse

- One existential evaluation owner handles SemiJoin, AntiJoin and every ExistsFilter term.
- For each input row, evaluate the right relation with the exact outer binding environment. EXISTS tests non-emptiness; NOT EXISTS tests emptiness. Evaluate effective term polarity once.
- Correlation edges declare required outer bindings; they are not equality join keys. The full right predicate governs membership.
- Preserve flat term/right-child pairing and authored term order. Do not rebuild a chain whose depth grows with term count.
- Preserve left multiplicity and stable relative order. Right duplicates do not multiply left rows; no incidental deduplication.
- Existing admitted rowset semi/anti/filter/projection kernels are the reuse boundary. Extract genuinely shared primitives into their owning module and migrate existing callers together; do not copy them into a second V3 engine. Existing equality join fast paths may be used only when the checked predicate proves the same semantics.
- Start with bounded parameterized evaluation. Caching or decorrelation requires an independent equivalence proof and a key containing the exact plan, outer binding values, generation and provider identity. No heuristic cache or optimizer is required for closure.

### Resource and receipt invariants

- One request root owns materialization permits, cumulative work accounting, deadline and cancellation checkpoints across producer, nested evaluation and output projection.
- Memory/row permits use the existing reserve-before-allocation mechanism and live lifetime accounting. Cumulative work is monotonic and is not released like live memory.
- Freeze what consumes work units: at least relation-row visits and scalar evaluation steps. Bound plan nodes, expression nodes, existential terms and nesting too. Correlated execution cannot activate without a finite admitted work bound.
- Missing/unsupported finite-work policy rejects at admission; do not silently reinterpret `UnboundedByPolicy` or substitute a magic limit.
- Budget/deadline/cancellation failure returns typed non-success. Never translate failure into empty right input, false EXISTS, true NOT EXISTS, partial success, or complete coverage.
- Receipt digests bind the actual compiled plan, admitted source/provider/generation identities and actual output rowset. After filtering, output is not the producer input. Retain permits for all live output/scratch and release scratch on success and failure.
- Preserve input truth/provenance/degradation metadata; do not gain a complete claim merely because a filter removed rows.
- If canonical receipt/admission digest bytes change, version that downstream domain and regenerate its vectors in the coordinated change. Do not bump the frontend contract unless its bytes or meaning change.

## Parallel ownership

Serial prerequisite: the integrator freezes the supported capability table, runtime plan API, source resolver API, error codes, execution-control API, input/output schema handling and digest changes. Writers do not invent competing interfaces.

### Lane A — QGLang source/capture owner

Own:

- `crates/quanta-gqlang-v2-binder/src/resolve.rs`
- Binder source-use/scope regression tests
- `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs`
- The relevant frontend test ledger row, if coverage changes

Implement:

1. Remove correlation emission from Composition LET relation-source resolution while retaining exact source resolution.
2. Keep real expression captures and noncapturing composition barriers.
3. Add permanent source regressions for one/two/mixed/nested EXISTS over the same LET, with and without actual outer-row path references.

Owner acceptance:

- The original valid counterexample compiles.
- Single EXISTS becomes SemiJoin, NOT EXISTS becomes AntiJoin, multiple terms become flat ExistsFilter.
- Bare LET scans have no spurious captures; actual outer expressions have exact captures.
- Shadowing, foreign composition LET use, forged/dropped captures and malformed sources still reject.
- Run affected Binder and pipeline tests. Repository-wide CI runs at serial integration, not independently in every lane.

Stop condition: if removing scan capture loses authority that is not already carried by the scan proof, report that exact producer/consumer mismatch. Do not weaken finalizers or add a compatibility payload.

### Lane B — Semantica relation-plan/execution owner

Own:

- `quanta-runtime-execution/src/gqlang/composition_plan_v3.rs`
- One runtime relation-plan compiler/evaluator module, if needed
- Shared rowset execution primitives and their affected existing callers
- `quanta-runtime-execution/src/budget_handle.rs` and execution-control plumbing needed by this slice
- Runtime plan/output/receipt digests and owner tests

Implement:

1. Replace Scan/identity-Project-only body classification with the frozen typed relation compiler.
2. Index expressions once, validate operator arity, term/right pairing, supported overloads and exact bindings; retain bounded traversal.
3. Add one existential evaluator with full parameterized right-predicate evaluation and existing admitted rowset primitives.
4. Enforce work/memory/depth/lifecycle controls inside loops and nested evaluation.
5. Produce and validate actual output schemas/rowsets and update receipt semantics.
6. Classify unsupported operator shape before checking Scan source authority. Preserve fail-closed error behavior.

Owner acceptance:

- All operators in the frozen slice execute; every other family/overload rejects before source acquisition.
- No repeated stage key overwrites operators, no missing binding becomes a default value, no polarity is applied twice.
- Empty/nonempty right inputs, correlated residual predicates, left/right duplicates, mixed terms and nested outer captures match independent rowset expectations.
- Exact-limit/over-limit, allocation rejection, cancellation and errors release scratch and produce no successful receipt.
- Existing kernel callers retain their previous supported behavior after any extraction.

Stop condition: a source or expression cannot be executed under issued authority, or an existing kernel has different semantics. Keep a typed rejection and hand the concrete missing owner contract to the integrator; do not switch to source reparsing or legacy planning.

### Lane C — Semantica issued ingress/public consumer owner

Own:

- `quanta-gqlang-v2-ingress-bridge/src/` and its feature-selected source tests
- `quanta-runtime-public-ingress-surface/src/admitted_execution_request_v3.rs`
- `quanta-runtime-public-ingress-surface/src/lib.rs`
- `quanta-runtime-public-ingress-surface/src/gqlang_execution_v3.rs`
- Public authority/source resolution, SDK handler routing and affected consumer tests

Implement:

1. Admit a typed Relation or Composition execution variant; retain exact selected item, issued generation/provider/source identities and the Lane B validated plan.
2. Resolve repeated Composition scans to the same admitted producer rowset. Resolve standalone canonical-files scans under issued host authority. Any additional source requires its own valid owner authority.
3. Compile once, acquire sources only after support validation, execute the frozen plan, and project its actual output.
4. Thread the same lifecycle/work/materialization root through acquisition, evaluator and public projection.
5. Preserve exact typed runtime reason codes through public error conversion; remove superseded Composition-only routing/classification in this route.
6. Keep one-column RepoPath public output validation based on the result schema, not a producer-field assumption.

Owner acceptance:

- Original source fixtures reach the intended runtime plan after Lane A lands.
- Standalone canonical-files Relation and Composition body share the compiler/evaluator.
- Foreign source generation/provider, swapped selection, revoked grant, unavailable relation and unsupported family reject before execution.
- No caller-supplied fixture authority is presented as issued product evidence.
- Bridge tests run with `v3-execution` enabled; zero selected/executed tests is not a passing proof.

Stop condition: issued source/schema/provider authority is absent. Keep rejection; do not admit Relation by deleting the Composition check alone.

### Lane D — serial integration and independent proof owner

This lane may prepare expected-result fixtures while A/B/C work. It owns cross-lane API edits, manifest/features, paired source freeze and final process tests; it does not modify A/B/C implementation files concurrently.

Independent fixture core:

| Case | Expected result / rejection |
| --- | --- |
| Left `[a,b,b,c]`, correlated right match only `b` | EXISTS `[b,b]`; NOT EXISTS `[a,c]` |
| Empty right | EXISTS empty; NOT EXISTS preserves all left rows |
| Nonempty uncorrelated right | EXISTS preserves all left rows; NOT EXISTS empty |
| Multiple/mixed terms | Intersection of each term's truth, preserving left duplicates/order |
| Outer alias shadowing / nested captures | Exact binding identity determines result, not spelling |
| Ordinary residual `false` / restrictive predicate | No identity pass-through result |
| Multiple flat terms plus residual Filter | Repeated stage provenance does not merge operators |
| Unsupported Sort/Group/general Join or scalar overload | Typed rejection before producer/source side effects |
| Budget/deadline/cancellation/provider failure on right | Typed non-success, never an empty-right answer |
| Wrong generation/provider/selected item | Admission rejection, no successful result/receipt |

Expected rows are handwritten or computed by a small test-only reference loop. It must not invoke production lowering, dispatcher, join helper or expression evaluator to build expectations.

Integration sequence:

1. Land A and freeze the QGLang commit; update Semantica's actual path dependency against that source.
2. Integrate B/C interfaces and implementations once; remove obsolete active routing/classification in the changed V3 route.
3. Run source -> compile -> select -> issued admission -> evaluate -> public result regressions for Relation and Composition. Test the enabled default consumer feature route, not only a private helper.
4. Run the smallest affected owner rails, then QGLang `just ci` once and the relevant Semantica prescribed CI rail with the execution features enabled. Record terminal selected/executed/pass counts and all skipped rails.
5. Commit/freeze both repositories; QGLang `just qualify-local` produces the new clean-HEAD frontend receipt.
6. Build the actual SDK/daemon candidate against that frozen pair and execute fresh-process source fixtures with issued authority. Record feature set, artifact identity and paired source digests.

Do not combine a frontend clean receipt and unrelated dirty-runtime unit tests into installed-product closure.

## Handoff and closure

Each lane hands off its source base, owned files, delta, support/rejection behavior, selected/executed/pass counts and unresolved consumer dependencies. Keep unrelated Semantica dirty work outside the integration candidate.

Completion requires all of the following:

- Confirmed frontend counterexamples compile with correct scan/capture facts.
- Supported existential results match independent expectations through the public route.
- Unsupported operators and absent authorities remain typed failures.
- Work/memory/lifecycle failures cannot produce existential success or a completed receipt.
- Actual output identity, schema, truth metadata and permits are preserved.
- Clean paired sources, required owner/CI rails and installed fresh-process evidence are recorded separately.

No compatibility shim, silent fallback, expression-table padding, ad-hoc source-name exception, or second EXISTS semantics implementation is part of this plan.
