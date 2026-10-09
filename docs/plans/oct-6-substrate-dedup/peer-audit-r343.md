# Independent audit of the substrate repair changes — R343

Observed: 2026-10-09. QGLang `2d5b1852f077cf5522e5420809e5ad7f597e07ad`;
Semantica `8217a38bf04d6c2fb2a535d43753408ac4621972` plus concurrent staged work.
This document records independent source review. It does not accept downstream
compiler, runtime, product or installation results.

## Findings and acceptance

| ID | Classification | Required completion |
| --- | --- | --- |
| W1 | Source lifetime violation in committed Java/Python workspace digest copying; runtime reproduction not run | External partial/completed text custody through the highest Source; preserve the first refusal and prove final refund |
| A2 | Integration gap between a staged DFG Source producer and existing ordinary PTA/fixture consumers; not a committed-HEAD Source regression | One supplier-owned read/binding contract, all reachable consumers connected, genuine Original positive and negative proof |
| T1 | Initial static false-pass possibility; successor pre-final regression authored by the existing topology lane | Execute the positive and strict negative tests through the actual pre-final owner; current execution remains unproved |

The R341 error-masking finding A1 is withdrawn. It confused
`with_lowering_unit_into_birth_v3` with the separate typed-consumer API.
Do not change the current error selection to repair A1.

## W1: preserve partial workspace text

Paths below are relative to Semantica's
`packages/analysis/quanta-v2/crates/quanta-adapters-parser/src/`.

- `semantic_scope_identity.rs:636-648` uses the return-shaped `copy_string_v1`.
- `language_host_adapter/semantic_module_v3/core/original_java_assembly_v3.rs`
  and `language_host_adapter/adapter_impl/native_typed_surface/retained_python_execution.rs`
  store only the successful returned workspace digest.
- Core `packages/core/codegraph-native-allocation-core/src/native_text_birth_v3.rs:45-93`
  can refuse after backing allocation or after pushing the copied bytes. Its
  local String then drops before the enclosing Source finishes.

Use the existing `workspace_owner_identity_namespace_digest_into_with_output_admission_v3`
and initially empty external partial/completed slots in the existing materialization
births. Keep DATA before funding fields in destruction order. If the applicable
Source contract requires the full allocator error, use the already exported
`copy_string_with_failure_into_v3` with an external failure bank; the current
into helper retains text but uses an internal empty allocator-error slot.

Acceptance requires real Original Java and Python inputs with multiroot axes,
refusal at the backing-birth and post-push checkpoints, retained partial text
until Source completion, first-cause preservation, exact last-drop refund, and
unchanged namespace bytes and successful output. No memory corruption, leak or
wrong successful result was reproduced by this audit.

R344 prepared a two-file caller patch using that existing helper. It adds
initially empty partial String fields to the existing Java/Python births and
leaves the completed workspace Option and namespace algorithm unchanged.
`git apply --check` passed against the recorded source. The patch is external
evidence, **PREPARED_NOT_APPLIED_NOT_BUILT**, not a repair in Semantica main.
Current live materialization claims overlap these files. Keep one implementation
owner and reconcile the actual hunks before applying it. The patch does not
resolve the helper's conditional full allocator-error custody requirement.

## A2: complete the existing supplier and consumer contract

The staged Java DFG producer publishes a Source member. Its ordinary member
reader returns typed `UnsupportedOperation` for that representation.

The reachable PTA route includes:

```text
original_java_workspace_v1
  -> original_java_evidence_v1
  -> original_binding_output_v3
  -> function_evidence_v1::collect_single_function_row_into_birth_v3
  -> ordinary with_row_v3
```

`function_evidence_v1.rs:166` precedes FunctionRef validation.
`native_normalized_attribution_generation_artifact_v2/function_ref_evidence_v3.rs:93`
and `dfg_declaration_index_v3.rs` also contain ordinary reads. Updating the
fixture count or the FunctionRef validator alone does not complete this route.

The external staged work briefly added and then removed a scalar lambda census
API during review. Do not treat that temporary API as a stable available contract.
Reuse the DFG owner's completed controlled evidence/binding read. Public CFG/facts
pair access alone does not supply the full semantic/binding evidence PTA needs.
Do not cast the Source payload, construct empty legacy DTOs, or replace authored
DFG evidence with the final PTA count. Parser nested-lambda counts were not
proved equivalent to exact DFG bindings.

The legacy exact fixture still feeds ordinary inventory to an Original issuer.
Connect the existing genuine inventory/provider; do not create a second supplier.
Acceptance requires one compatible source/features tuple with real void, alias
and lambda positives; declaration/function identity, duplicate/missing/foreign
rejection, charged work and cancellation; independent authored/final function-ref
census; and explicit final refs=0 for VoidReturnOnly. Unsupported nested/captured
cases must remain typed refusals until their contract is supported and proved.

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
patch and its remaining acceptance conditions.

See [the current execution plan](parallel-dispatch.md) for the existing
integration and owner queue. This peer audit does not rewrite that queue or
close the active runtime tickets.
