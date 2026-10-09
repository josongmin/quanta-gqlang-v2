# Independent audit of the substrate repair changes — R343

Observed: 2026-10-09. QGLang `2d5b1852f077cf5522e5420809e5ad7f597e07ad`;
Semantica `8217a38bf04d6c2fb2a535d43753408ac4621972` plus concurrent staged work.
This document records independent source review. It does not accept downstream
compiler, runtime, product or installation results.

## Findings and acceptance

| ID | Classification | Required completion |
| --- | --- | --- |
| W1 | Committed-baseline lifetime defect; R348 repair applied to dirty source, not qualified | External text and full allocator-cause custody, genuine late-refusal and exact refund proof |
| A2 | Staged Source integration partly connected for ExactZero; final-generation consumer still ordinary-only | One supplier-owned contract, all reachable consumers connected, genuine positive and negative proof |
| T2 | Confirmed R346 defect; existing owner's R347 source correction independently reviewed | Execute genuine Source-zero positive and rejection/finisher regression; runtime not run |
| T1 | Initial static false-pass possibility; successor pre-final regression authored by the existing topology lane | Execute the positive and strict negative tests through the actual pre-final owner; current execution remains unproved |

### Current checkpoint — R348

Readback: QGLang `30c0fad3a9d835dec07ecba60f3e663f33d7a47a`;
Semantica `8217a38bf04d6c2fb2a535d43753408ac4621972` plus concurrent changes.
The W1 successor changes are applied in fourteen Semantica files under reconciled
claim `OCT9-GQLANG-WORKSPACE-CUSTODY-PEER-R345`. This chat did not stage, commit
or push Semantica, or run its Rust compiler, tests, QBC or runtime.

W1 has independent source review and static checks. A2 remains source work;
T1, T2 and W1 still require executed owner proof. Existing Source ExactZero support
does not support nonzero lambda references or establish final generation success.

The R341 error-masking finding A1 is withdrawn. It confused
`with_lowering_unit_into_birth_v3` with the separate typed-consumer API.
Do not change the current error selection to repair A1.

## W1: preserve partial workspace text

Baseline paths below are relative to Semantica's
`packages/analysis/quanta-v2/crates/quanta-adapters-parser/src/`.

- `semantic_scope_identity.rs:636-648` uses the return-shaped `copy_string_v1`.
- `language_host_adapter/semantic_module_v3/core/original_java_assembly_v3.rs`
  and `language_host_adapter/adapter_impl/native_typed_surface/retained_python_execution.rs`
  store only the successful returned workspace digest.
- Core `packages/core/codegraph-native-allocation-core/src/native_text_birth_v3.rs:45-93`
  can refuse after backing allocation or after pushing the copied bytes. Its
  local String then drops before the enclosing Source finishes.

The repair reuses `workspace_owner_identity_namespace_digest_into_with_output_admission_v3`
and initially empty external partial/completed slots in the existing materialization
births. Keep DATA before funding fields in destruction order. R348 extends this
same helper to accept an external `Option<TryReserveError>` and invokes the already
exported Core `copy_string_with_failure_into_v3`. An occupied failure bank is
rejected before axes/hash work, preserving the recorded first refusal.

Acceptance requires real Original Java and Python inputs with multiroot axes,
refusal at the backing-birth and post-push checkpoints, retained partial text
until Source completion, first-cause preservation, exact last-drop refund, and
unchanged namespace bytes and successful output. No memory corruption, leak or
wrong successful result was reproduced by this audit.

R344 prepared a two-file caller patch using that existing helper. It added
initially empty partial String fields to the existing Java/Python births and
leaves the completed workspace Option and namespace algorithm unchanged.
`git apply --check` passed against the recorded source. The patch is external
evidence at that checkpoint, **PREPARED_NOT_APPLIED_NOT_BUILT**. R345 subsequently
applied its two caller changes. R348 supersedes that prepared-only state with the
full coupled source repair below; Semantica main integration remains unconfirmed.

### Applied successor — R348

- Four direct production helper callers now pass partial text, completed text and
  a full allocator-error bank: Java materialization, Python materialization,
  Python import bindings and Python import augmentation.
- Five existing births retain those banks. Both manual owner-seal/provider-frame
  constructors initialize them to None. The augmentation transaction, owner seal,
  provider completion and existing integration-test caller are changed together.
- One existing public augmentation method gains the external bank in its tuple.
  All repository callers were traced and updated; this is a Rust source signature
  change, with no frontend wire DTO, new provider, crate or engine.
- The public `with_original_python_semantic_modules_v3` formerly created its
  materialization birth inside an outer issued-tree read. It now creates the same
  birth before that read; a private into-slots helper parks results/errors and
  drains them only after the outer finisher. It adds no cohort move or validation.
- The existing Original namespace test keeps its independent literal BLAKE3
  preimage and adds an actual capacity-overflow `TryReserveError` bank with a
  no-new-work rejection oracle. This does not force physical failure of the
  fixed 64-byte namespace allocation.

Static results: AST parse and rustfmt PASS for 14 paths; scoped LLM and God gates
PASS for their 13 selected paths; diff check PASS. God reports five pre-existing
stale entries, with no new allowances or baseline change. Initial transaction
size failure was repaired by sharing its repeated tuple signature as one local
transparent alias; the alias has the same visibility as its two parent-visible
functions. All five static checks passed for that final one-file delta.

Independent wrapper and full-cause reviews found no new confirmed defect in the
applied scope. Runtime reproduction, Rust type checking, actual late-refusal
custody and last-drop refund remain **NOT_RUN**. A result-only cancellation test
would not detect the old early-drop bug: observe retained DATA/grants at the outer
finisher and verify the refund after their final drop.

## A2: complete the existing supplier and consumer contract

The staged Java DFG producer publishes a Source member. Its ordinary member
reader returns typed `UnsupportedOperation` for that representation.

The reachable PTA route includes:

```text
original_java_workspace_v1
  -> original_java_evidence_v1
  -> original_binding_output_v3
  -> function_evidence_v1::collect_single_function_row_into_birth_v3
  -> with_identity_row_v3
```

The initial `function_evidence_v1.rs:166` ordinary-read blocker has been changed
in concurrent source to controlled identity reading. The declaration ordinal
index and FunctionRef preflight also now accept the supplier's sealed ExactZero
subset. Those changes supersede the initial blanket ordinary-read claim.

The final generation still calls
`native_occurrence_admission_v1.rs:73-75 -> declaration_by_dfg_key_v1` unconditionally.
That function at `dfg_declaration_index_v3.rs:375-399` uses the ordinary-only
`try_visit_rows_v3`, so a Source member still reaches a typed refusal. Its owner
must define Source index semantics; fabricated legacy DFG rows or an unverified
empty-index shortcut cannot close this contract.

The external staged work briefly added and then removed a scalar lambda census
API during review. Do not treat that temporary API as a stable available contract.
Reuse the DFG owner's completed controlled evidence/binding read. Public CFG/facts
pair access alone does not supply the full semantic/binding evidence PTA needs.
Do not cast the Source payload, construct empty legacy DTOs, or replace authored
DFG evidence with the final PTA count. Parser nested-lambda counts were not
proved equivalent to exact DFG bindings.

The legacy full exact fixture still feeds ordinary inventory to an Original issuer
(`owner_stages_v1.rs:434-441`); the new Source fixture tests cover DFG issuance and
controlled zero reading, not the full topology/final-generation route.
Connect the existing genuine inventory/provider; do not create a second supplier.
Acceptance requires one compatible source/features tuple with real void, alias
and lambda positives; declaration/function identity, duplicate/missing/foreign
rejection, charged work and cancellation; independent authored/final function-ref
census; and explicit final refs=0 for VoidReturnOnly. Unsupported nested/captured
cases must remain typed refusals until their contract is supported and proved.

## T2: Source-zero completion state in the R346 patch

`exact_native_generation_fixture_v1/topology_v3.rs:193-201` returns Ok after the
controlled Source reader returns Some(0). That branch never starts the ordinary
`visit_birth_v3`. However the outer success condition at `:57-66` unconditionally
requires `visit_birth_v3.completed_v3()`. Its empty birth remains incomplete, so
a valid Source-zero result falls through to `CallableCensusMismatch`.

This is a confirmed source-logic defect in the receiving patch, independently of
the remaining upstream fixture and final-generation blockers. It was not run.
The repair must retain explicit verified Source-zero state only
after the controlled read returns success, then accept either that state or the
completed ordinary visit, retaining every other completion/error check. Source
Err, mixed-cohort refusal and ordinary None must not bypass their normal gates.
Use a genuine Source success regression that fails on the former unstarted-birth fallback;
the two authored negatives alone cannot establish this positive path.

### Existing-owner correction — R349 peer readback

The topology owner has now added `source_function_refs_read_v3` to its existing
DATA. Default leaves it false; its only true assignment follows successful
controlled Source/output reading. The outer condition accepts this observed
Source read or completed ordinary visitation, retaining the highest Source,
lower/census completion and error checks. Source Err exits before the assignment;
ordinary None still requires the visitor. No unused birth is marked complete.

The initial peer read raced this edit and has no successor verdict. The root
review uses one saved byte snapshot, SHA256
`63bb5c6efcac207b93f6d46ebb08e5133c409bbc9b536c0c461c0795a6d75e82`,
in `r349-topology-single-read.rs` and `r349-topology-root-successor-review.json`.
The identified source defect is repaired in that snapshot. Actual owner tests,
QBC and final-generation integration remain **NOT_RUN**, so this is source review
of the correction, not executed acceptance.

## T1: distinguish pre-final rejection from later rejection

`exact_native_generation_fixture_v1/topology_v3/tests.rs:10-31` expects a no-op
input requested as `ExactFunctionReferences(1)` to return
`UnexpectedRelationTopology` with counts `(0, 0, 1, 0)`.
`owner_stages_v1.rs:472-500` can produce the same variant and tuple from its
later final-generation check. The test therefore does not independently prove
that the earlier topology rejection was reached and propagated.

Use a test-only preparation route to call the existing
`issue_and_validate_native_parts_v1` before the final checker and assert its
actual return. Reuse the genuine fixture preparation and the existing Source
owner. Preserve the independent final checker. No public phase field, second
provider or production counter is needed.

A test-only TLS refusal counter is insufficient by itself: topology may return
Err and increment the counter, its caller may ignore that Err, and the final
checker may return the same expected error. The counter and literal error both
match in that static counterexample. Test the returned pre-final result directly.

The lambda-to-VoidReturnOnly negative does not enter the final checker's
ExactFunctionReferences branch, so that particular false-pass path does not
apply. Neither negative has current executed owner proof in this audit.

### Successor source review — R344

The existing topology lane has now changed its tests to prepare the genuine
Original inventory and call a test-only
`ExactJavaOriginalInventoryFixtureV3::validate_native_topology_v3` accessor.
The accessor directly returns `issue_and_validate_native_parts_v1(...).map(|_| ())`;
it does not invoke the final-generation checker or introduce an extra Source.
The original negative literal errors remain. New no-op/VoidReturnOnly and
returning-lambda/ExactFunctionReferences(1) positives use the same preparation.

This removes the identified later-checker substitution in the test source.
Skipping topology, converting its refusal to Ok, or ignoring its Err causes the
direct negative expectation to receive Ok. Upstream refusal still fails the
required topology variant/count. This is a static counterfactual review, not
an executed mutation test or runtime verdict. Supplier compatibility and all
four actual test executions remain open.

## Ownership and verification boundaries

- The existing `(3)` topology lane owns the consumer and pre-final regression.
- Existing DFG/PTA/Java owners own Source evidence, binding and genuine inventory.
- W1 overlaps current materialization claims. Retain one implementation owner
  for those hunks; general supplier activity does not establish that W1 is fixed.
- Initial D1-D7 removals are already in observed Semantica main. These findings
  do not reopen seven duplicate implementations or create a new engine task.
- Borrowed scope views, terminal guards and required reexport features had no
  additional demonstrated source defect in the inspected changes.
- Rust tests, QBC publication, runtime and installed outcomes remain **NOT_RUN**
  for this audit. Registered tests and static reads are not executed results.
- Existing frontend `GREEN@99fc50d` stays bound to its original clean source.
  A new documentation commit is not new frontend or Semantica qualification.

Detailed external evidence is under
`/Users/songmin/.codex/task-evidence/oct6-substrate-dedup/recovery-20261006/query-owner-build-r1/`:
`r343-peer-own-changes-audit.md`, the three independent lane JSON reports, and
the peer postflight/delta readbacks. Moving foreign staged inputs and their
temporary API removal are recorded separately from stable consumer evidence.
The later `r344-t1-peer-review.json` records the direct pre-final test source;
`r344-workspace-custody-prepared.patch` and its JSON record the unapplied caller
patch at that historical checkpoint. Current source is recorded by
`r348-workspace-owned.patch`, `r348-workspace-source-closeout.json`, the two R348
postimage peer reviews and final static-check logs. `r347-source-zero-peer-review.json`
records T2 and the remaining A2 final-generation/fixture paths. Prepared, applied,
executed and integrated states must remain distinct.

See [the current execution plan](parallel-dispatch.md) for the existing
integration and owner queue. This peer audit does not rewrite that queue or
close the active runtime tickets.
