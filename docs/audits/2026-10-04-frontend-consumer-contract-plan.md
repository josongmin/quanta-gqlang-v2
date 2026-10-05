# Frontend / Semantica contract repair plan

## Current status — 2026-10-05 22:49 KST

Current producer source: `main@1ddeaacaa754c0d432c88dc25ba3feac83f47c1c`
before this documentation checkpoint. Consumer source:
`main@3f037ea04eecd80606f045ad9b3a7709c5a17ee7`.
The current disposition and source/receipt boundaries are maintained in the
[latest session audit](2026-10-05-current-source-audit-action-plan.md).
The work packages and older verification entries below retain their original
source identities; they do not qualify the current shared checkout.

| Scope | Current disposition |
| --- | --- |
| Producer structure, request owners, local size/dependency/MSRV gates | Integrated in `e598a1d`; clean source867 local CI 1514/1514 and bounded fuzz PASS. Tracked code/tool inputs are unchanged at the current producer HEAD. |
| Graph ingress profile, schema/vector pins, EXISTS independent results, conformance | Main checkpoints recorded in the latest audit. Rename is included in 19 facade families. No schema change beyond the intended 3/8/8 contract. |
| Remaining confirmed code gap | Native Match canonical length and encoding traverse the artifact without original-request work admission. Existing memory reservations and later hash work do not cover these passes. |
| PTA bootstrap investigation | Old Mac/Linux process attempts refused before query. Current cohort retains the seed consumption owner; current-source bootstrap resolution is unverified. Identify the exact refusal before deciding another repair. |
| Process fixture integration | Candidate Graph/CHECK inputs and Mac Graph refusal regression remain outside main pending actual-query GREEN on one clean tuple. |
| Linux installation | Generic aarch64 product GREEN does not satisfy the signed x86_64 installed profile/root/slot/approval prerequisites. This is an installation gap, not an established trust-code defect. |

CI is local. New engines, compilers, physical optimizers, SMT and production
dependencies are outside this maintenance scope. Broader downstream AG5/IRC
feature and release work remains in those owners' plans.

## Objective and scope

Make the currently supported source subset survive frontend compilation,
Semantica selection/admission, and execution with the authored meaning intact.
Complete four changes: Graph admission alignment, producer/consumer pin
alignment, conjunctive EXISTS result coverage, and facade conformance coverage.
Use the existing frontend, DTO encoding, singleton graph traversal, and test
targets. Runtime implementation and proof remain owned by Semantica.

This plan changes no DTO fields or encoding tags; source schema 3 and
contract/qualification schema 8 remain the intended contract. It does not add
a query engine, optimizer, rule interpreter, SMT solver, or production dependency.
The added CHECK regression uses three existing workspace crates as test-only
direct dependencies; both consumer workspace lockfiles record those edges.
The captured-Python owner regression adds one further existing workspace crate
as a test-only direct edge. There is no new external or production dependency.

## Current action list

| Priority / kind | Work | Acceptance |
| --- | --- | --- |
| P0 / code | Meter Native Match canonical counting and encoding | Original request control charges visits/payload before work; exact/one-short and no-transport regressions; existing 4 MiB cap and typed refusal retained. Coordinate parser/foundation writers. |
| P0 / diagnosis | Resolve Product PTA bootstrap refusal | Identify the first refused boundary and verify the already integrated seed-lifetime change on current clean source; repair only a remaining reached failure. |
| P0 / installation | Establish canonical Linux installed inputs | Signed x86_64 profile, root-owned immutable slots, independent approval/key and active generation from the release owner. |
| P1 / test integration | Prove actual Graph/CHECK and merge reviewed fixtures | One clean producer/consumer/index tuple, immutable product artifacts, actual query positive/rejection results; merge only GREEN scope and preserve historical RED attempts. |

The producer structural and contract repairs are complete in the scoped main
checkpoint. Local fuzz scheduling is an operational choice. Authored
`result_shape` remains typed unsupported. DTO definitions stay in their owner;
the facade remains the sole downstream seam. All current proof details live
in the linked session audit rather than a second manual result matrix here.

## Disposition of the original ten audit actions

| Original action | Current evidence / next action |
| --- | --- |
| Host `just ci` in GHA | Use local `just ci`, as requested. Existing untracked CircleCI config/install scripts are outside this plan and must not be included in its owned commit. |
| Include `clippy-msrv` in static/CI | Integrated in producer main; both current/MSRV Clippy passed clean source867 local CI. |
| Split `document.rs` | Owner modules and typed preparation/inline-view/source-order helpers are integrated. The orchestration function has no production function-size exception. |
| Split `execution.rs` / move DTO | Encoding, projection, vocabulary and tests are separate. Keep the existing public facade; a reverse pipeline-to-facade dependency would cycle. |
| Split three mega integration tests | Three harnesses now mount thematic children; measured sizes are listed below. Keep test-only module guards. |
| Schedule fuzz smoke | The same clean source867 completed a 60-second local smoke with exit0/crash0. Keep it bounded and separate from deterministic CI; periodic local scheduling is optional. |
| Split `types/check.rs` / `solver.rs` | Inputs/value checking and candidate owners are extracted. Current size policy has no legacy exceptions. |
| Expand closed-authority examples | Conformance checks all 18 shipped examples under test-owned closed authority: ten plans, seven non-plan outcomes, one exact unsupported-profile rejection. Rename preview/apply are added to the facade family corpus; shipping a Rename example is optional documentation work. |
| Document Execution DTO 1:1 | The contract document contains a stored-field/variant map and canonical stream clauses with independent encoding vectors. The map is manual; code/doc drift is not automatically proved absent. Update it whenever a public DTO changes. |
| Clean `sema/test_support.rs` | Common helpers and family fixtures/owner tests are separated; the largest current helpers are below the test-file limit. |

Current HIR and sema module comments describe the live parser and transaction
owners; the old no-parser/root-family-RED comments are removed. A connected
compile pipeline does not establish support for every authored form: all
authored `result_shape` values reject in sema, and the specialized
`admit_show_planned_source_v2` always returns typed incomplete diagnostics (or
source-limit refusal) until parser diagnostic allocations have an issued
request-root debit. The ordinary closed source compiler is a separate entry.
The Pipeline staged syntax → declaration → plan → Execution DTO path is live;
its owners and one-shot refusal have positive/short/foreign/drop regressions.
Only the syntax entry is exported by the downstream facade. Keep these
limitations explicit; implementing either unsupported form requires an
owner-scoped consumer requirement and acceptance criteria.

## Historical audit baseline and evidence boundaries

- QGLang: `main`, HEAD `9060e46997dd9c62080a18bdfae06f8c162442e2`, shared dirty tree.
- QGLang pre-plan local source snapshot:
  `75a8dacdf8fed769f2e57c36fa10f507d1a6560f160f03f1fd21cd0a7e597326`.
  This includes the existing overlay and predates this plan file. It is not a
  qualification receipt or the eventual implementation snapshot.
- Semantica: `main`, started at `255a832eecf771ca24e87bff06f52511b612e89c`;
  subsequently observed `d410488f83c8a457bf248184195829dacaa454f0`.
  Inspected consumer/host/test/gate paths remained clean. HEAD moved through
  concurrent work; the audited target paths did not change in that interval.
- Existing dirty QGLang paths include frontend-contract, pipeline splits,
  sema authority, conformance tests, and language/contract documents. Preserve
  their current changes. Neither shared checkout is a clean release baseline.
- Three parallel read-only audits covered graph, encoding/consumer custody,
  and relation execution; the primary audit covered conformance and rejection.
- At the audit baseline, only source reads and one EXISTS QBC `owner resolve`
  were executed. Implementation results are recorded below; this baseline is
  historical, not the current verification status.

| ID | Classification | Baseline evidence |
| --- | --- | --- |
| G1 | Reachable admission mismatch | Semantica `quanta-runtime-public-ingress-surface/src/lib.rs:511` selects standard budget; QGLang `sema/src/authority.rs:661` sets witness cap 10,000; `exec-host-spi/src/analysis_host.rs:402` accepts only 1. |
| G2 | Reachable policy mismatch | QGLang `logic/src/graph.rs:194` seals Path=NodeSequence and Reachability=Endpoints; the same host validator accepts only Endpoints. |
| V1 | Reachable stale golden | Producer TSV `composition_limit_2`, line 9, is schema 8; Semantica `admitted_execution_request_v3.rs:1781` expects schema 7. Its Cargo path dependency resolves the live sibling producer. |
| V2 | Qualification pin drift | Semantica `gate-gqlang-v2-semantica-closeout.sh:930-962` pins older frontend/pipeline trees and TSV hash; its digest needles also differ from the consumer golden. |
| E1 | Result-proof gap | `gqlang_relation_execution_v3.rs:177` tests two true EXISTS terms and empty outer input, but not one false term with nonempty outer input. No incorrect runtime result was observed. |
| C1 | Registry-proof gap | Conformance declares every public source family but registers 18 without Rename; the parser exposes Command and pipeline Rename preview/apply tests already exist. |
| C2 | Rejection-proof gap | `sema/src/document/query_materialization.rs:31` rejects every authored result shape. Parser shape tests exist; the facade registry has no matching closed-compile rejection case. |

## Work packages

### W0 — Establish source ownership and execution inputs

1. Re-read HEAD, relevant dirty hunks, manifests, test features, and source
   digests immediately before implementation and before each proof.
2. Preserve shared overlays. Define the owned hunks, then inspect staged diffs
   if commits are requested. Do not stage unrelated changes or reset a checkout.
3. Resolve the existing QBC owner requests and compatible warm lanes before
   expensive consumer tests. A test filter narrows execution, not compilation.
4. Keep correctness tests on the actual transitive producer/consumer source.
   On relevant source drift, re-resolve/re-run affected proof.

Output: recorded source pair, owned change list, reviewed compile topology.

### W1 — Align the singleton Graph profile and policy

Purpose: standalone Path/Reachability pass public admission and reach the
existing bounded traversal with their complete sealed policy.

Production changes, all in Semantica:

1. Change `quanta-runtime-public-ingress-surface/src/lib.rs::frontend_budget_v3`
   to construct the Semantica singleton-witness profile using the existing
   `FrontendBudgetV2::try_new`: copy the other 21 standard values and set
   `max_witnesses=1`. Keep root classification and compilation on that same
   budget. The profile is selected before the DTO is sealed.
2. Change `quanta-gqlang-v2-exec-host-spi/src/analysis_host.rs` to validate
   dedup by kind: Path=NodeSequence, Reachability=Endpoints. Keep cap=1,
   positive bounded depth, Forward/None, Cyclic, ShortestThenLexical, exact
   Entity endpoints, and empty summary facets.
3. Preserve the existing physical singleton BFS in
   `quanta-cli-server/src/production_analysis_host_graph_execution.rs`.
   Its endpoint policy returns one canonical source/sink witness; prove that
   this satisfies the accepted Path policy with cap 1. If that equivalence
   fails, repair the adapter policy rather than rewrite the sealed query.

Compatibility and failure invariants:

- QGLang `standard_v2()` stays at 10,000; the host's ingress profile is local.
- No DTO field/tag or digest schema revision is needed. Graph contract and
  qualification values change because their encoded witness bound changes.
- Budget itself is not directly encoded by `digest_qualification`; it binds
  source and contract digests. A graph-free relation/composition should retain
  its bytes/digests when only the unused witness cap changes. Verify that
  invariant instead of regenerating unrelated goldens.
- Do not clamp a sealed 10,000-witness DTO, silently discard graph policies,
  or accept a larger witness contract without an implementing traversal.
- Composition Graph producers and nested CHECK Graph remain typed unsupported
  at `composition_plan_v3.rs:408` and `admitted_execution_request_v3.rs:194`.
- Preserve generation/source/provider custody, incomplete graph rejection,
  and work/memory charging. Existing traversal tests cover these owners.

Focused proof:

- Extend `public_source_family_contract_v3.rs` to assert exact sealed cap,
  kind-dependent dedup, and admitted plan for both Path/Reachability using
  the actual ingress helper and issued catalog.
- Reconcile `graph_frontend_references_bind_plan_but_not_runtime_authority_v3`:
  its direct standard-budget success expectation currently contradicts
  admission. Keep the standard profile as an explicit cap-mismatch rejection
  and retain the plan-digest/custody assertions under the supported profile.
- Cover standard cap 10,000 rejection, authored unsupported kinds/directions,
  and foreign catalog selectors. Assert the emitted kind-dependent dedup values.
  Malformed dedup pairs are unrepresentable through the sealed public DTO;
  do not add a public mutation/test constructor merely to fabricate them.
  Constructor/parser owners retain zero-cap/depth rejection.
- Check graph-free digest parity across the two budgets.
- Run the existing installed CLI/daemon test
  `daemon_native_gqlang_graph_path_and_reachability_use_issued_source_v3`.
  Use an independent expected canonical path; preserve its existing empty
  result and exhausted-budget cases. Existing BFS diamond tests need no copy.

Completion: exact fields and rejection behavior pass owner proof; both public
queries return the expected path result through the installed process boundary.
Owner proof alone does not close the process claim.

### W2 — Align schema-8 consumers and source pins

Purpose: test and qualify the actual producer/consumer pair without stale
goldens or a gate that admits a different source tree.

1. After W1/W4 and relevant producer overlays are reviewed, freeze the final
   producer contract closure. Coordinate clean committed source for pinning.
2. Verify the existing producer stream oracle and canonical vectors. The
   schema-8 `composition_limit_2` baseline is:
   - source: `d47176ffd6ba1ce41f1e6799c4228e3261e634cea9d3413b5c626a4ed5ff4731`
   - contract: `684e67231b434a0f0e6f9a360e8eb22cf5209d7fbe600a97963b48d59eae25dc`
   - qualification: `1cdf73b2b4412118b78a7c92cd43a06ac756c6c4a1694e063955259fa0f48569`
3. Update Semantica's composition golden values/comment in
   `admitted_execution_request_v3.rs`; assert schema versions as well as values.
   Do not replace values merely because the SUT printed new output.
4. Update the closeout gate's frontend/pipeline Git trees, vector SHA-256,
   and consumer needles together, using the final committed producer source.
   Retain dirty-closure rejection and exact matching. Never pin the temporary
   overlay or refresh pins before the producer proof/source is final.
5. Run the exact consumer golden and existing digest/item/generation rejection
   rails. Recheck the external workspace dependency custody.
6. Run the scoped `--ticket M23-120` cutover gate after its prerequisites are
   ready. It avoids the known `--all` host-matrix stop and performs static pin
   checks, Python lints/tests, and offline Cargo metadata. It does not compile
   or execute Rust; the exact consumer proof in step 5 remains separate.

Completion: schema values agree, exact consumer tests pass against the final
producer, and the scoped gate verifies its committed trees/vector and consumer
digest needles. Full joint/product closeout is separately accounted below.

### W3 — Prove conjunctive EXISTS results

Purpose: detect OR substitution and either-term omission during refactoring.
Production changes: none expected.

Extend the existing `flat_mixed_polarity_and_empty_source` test in Semantica
`quanta-runtime-execution/tests/gqlang_relation_execution_v3.rs`.
Use nonempty outer/source rows `["a"]` and literal expected results `[]`:

```text
from sampled s | where exists (from sampled t | where true) and not (exists (from sampled u | where true)) | project s.path as path
from sampled s | where exists (from sampled t | where false) and not (exists (from sampled u | where false)) | project s.path as path
```

The cases are `(true,false)` and `(false,true)`. They both lower to the existing
multi-term ExistsFilter, then `Op::Exists`. The first catches dropping the
second term, the second catches dropping the first, and both distinguish OR.
Retain the existing positive/empty fixture. Duplicate rows, correlations,
nested capture, and standalone polarity already have result tests.

Reviewed QBC request (resolve again on implementation inputs):

```sh
scripts/quanta-build-cli owner resolve \
  --package quanta-runtime-execution --execution-kind test \
  --target-kind integration --target-name gqlang_relation_execution_v3 \
  --selector-mode exact --selector flat_mixed_polarity_and_empty_source \
  --compile-policy feature-isolation:quanta-runtime-execution.no-default.6aeb25727949
```

The read-only resolve succeeded during this audit. Its feature closure is
`gqlang-relational-composition-exec-v3`. Execute the same request through
`owner run` on a compatible existing lane, with a nonzero executed count and
source-bound receipt. No NULL, Sort, Group, or generic evaluator extension.

Completion: both literal empty-row expectations and existing positives pass.

### W4 — Complete the registered facade source coverage

Purpose: registered source-family claims include Rename, and parsed-but-
unsupported result shapes cannot silently produce an execution document.

QGLang changes:

1. Add Rename to `conformance/src/lib.rs` (19 registered families including
   Composition), with a canonical preview source and CommandReceipt/ReadOnly
   expected disposition. Register one closed exact symbol target in the
   existing test authority builder.
2. Update `conformance/tests/self_validation.rs`'s count and Command family
   mapping. Assert the Command payload and preview/apply phase, effect, target,
   snapshot, and token through facade-only imports. Keep apply a source-intent
   proof; it grants no runtime mutation authority.
3. Add a dedicated apply case beside the registry loop. Do not append it to
   supplemental sources whose current contract is non-executable artifacts.
4. Classify registry and supplemental sources through the public root
   classifier with an exhaustive closed-family mapping. Cover Command and
   retain Declaration/Rule supplemental dispositions. Describe the coverage
   as registered cases, not proof of every possible language combination.
5. Add closed-facade result-shape rejection cases for a leaf, flat record, and
   nested record/list, with exact typed diagnostic code/stage/source span.
   Use valid parse fixtures and a shape-free positive control; exercise a
   Composition query context to catch silent shape omission at that caller.
   Existing parser depth/duplicate/empty-shape tests are not duplicated.
6. Correct the 'nested only' wording in the language/contract docs to describe
   all authored result shapes. Keep behavior and diagnostic encoding stable.
   Update registered counts in Cargo description and test ledger; distinguish
   those counts from the unchanged 18-file shipped example corpus.

Files: conformance lib/tests/Cargo.toml, `docs/language/README.md`,
`docs/language/test_hellgate_ledger.md`, and
`docs/contracts/frontend-execution-v3.md`. Inspect example-index count wording
for taxonomy drift, without adding a mutation example or altering its corpus.

Focused rail:

```sh
cargo test -p quanta-gqlang-v2-conformance --test self_validation --locked --offline
```

Completion: 19 registered families, exact Rename preview/apply DTO intentions,
open missing-authority behavior, and precise unsupported shape rejection pass.

## Order, verification cost, and closure

1. W0; then author W1 and W3/W4 with separate file ownership where useful.
2. Resolve and run the smallest meaningful owner targets after static review.
   Fix an observed mismatch at its owner; never weaken a test/gate to pass.
3. Finish producer changes and source review. Run QGLang `just ci` once for
   local completion, without pre-running its leaves. For clean-HEAD
   qualification use `just qualify-local` on a clean reviewed committed
   checkout; it executes CI itself, so do not mechanically duplicate both.
4. Finalize W2 pins after producer source is committed; run focused consumer
   proof and the applicable scoped cutover gate on the final pair.
5. Run Graph process proof with source-matching immutable binaries and manifest.
   First build/install or verify the receipt-bound artifact set required by
   `require_configured_process_artifacts_for_promotion_v1`: bundle root,
   CLI/server/SQL-worker binary paths and their three SHA-256 values, server
   build root, and build receipt path. Required inputs are
   `QUANTA_RELEASE_BUNDLE_ROOT`, `QUANTA_CLI_BIN`, `QUANTA_CLI_SERVER_BIN`,
   `QUANTA_SQL_RULEPACK_WORKER_BIN`, `QUANTA_CLI_SERVER_BUILD_ROOT`,
   `QUANTA_CLI_SERVER_BUILD_RECEIPT_PATH`, `QUANTA_CLI_BIN_SHA256`,
   `QUANTA_CLI_SERVER_BIN_SHA256`, and `QUANTA_SQL_RULEPACK_WORKER_BIN_SHA256`.
   Missing input is a hard stop, with no source-tree fallback.
   The registered recipe is
   `quanta-cli-product.release-full.process-test-target` (dist-full +
   gqlang-process-proof); it runs the whole daemon test target. Resolve the
   product/owner route first; this test recipe does not itself establish the
   installed artifact tuple. Do not assume an exact filter is compile isolation.
6. Collect command, source pair/digests, toolchain/features, executed counts,
   artifact identities, result, and residuals. Distinguish code completion,
   owner proof, process proof, and qualification.

All verification is local. Source edits are small; consumer compilation,
installed artifacts/process proof, and clean source coordination may dominate
elapsed time. No wall-time estimate was measured in this audit.

## Separate full-closeout blockers

1. Semantica `gate-gqlang-v2-semantica-closeout.sh:1447-1460` unconditionally
   fails after the supported SDK read-query slice, and `--all` invokes that
   function first (`:1759`). The S21 analysis/command-apply **public process
   matrix** is outside the four repairs. Rename admission and execution code
   exists; the sentinel is not evidence that every such implementation is
   absent. Preserve it until the required live owner proof is established.
2. The joint verifier requires rail `gqlang-v2-ingress-bridge-joint-v1`, exact
   command, 13 witnesses, QBC result/source custody, and external workspace
   custody. `produce_owner_local_exact_rail_receipts.sh` does not register this
   rail. The generic writer supports QBC-bound receipts, but a working joint
   producer route was not verified. Resolve/register that route and test its
   stale/empty/wrong-source rejection before a joint qualification claim.

3. The full GQLang provider process class
   `quanta-cli-server.no-default.61b2510e7d2a` currently fails compilation:
   the dense SAMPLE test caller in `server_socket_daemon_e2e.rs` and the adapter
   unit test still pass two callbacks to the owner's three-callback fixture.
   Its publication callback must produce real native SDK publication, exact
   snapshot and persisted Delivered generation. The correct procedure exists
   only in a private SDK lib-test helper. Extract a narrow proof-only SDK helper
   and update both callers before claiming full provider closure proof. Keep
   their current rejection assertions and generation validation. A dummy
   callback or invented generation would invalidate the test precondition.

Four-item scoped completion does not imply `--all` passes. Preserve these
blockers as separate work; do not implement unrelated engine/product features
or fabricate receipts to turn the full gate green.


## Reference patterns

- [Substrait technology principles](https://substrait.io/spec/technology_principles/):
  explicit producer/consumer subsets and compatibility boundaries.
- [DataFusion testing](https://datafusion.apache.org/contributor-guide/testing.html):
  user-visible results and smallest meaningful owner proof before broad gates.
- [Joern data-flow steps](https://docs.joern.io/cpgql/data-flow-steps/):
  distinguish source reachability from path evidence.

These patterns guide the existing owners; none requires importing the engines.

## Historical broader structural audit and current disposition

The earlier large-file numbers describe the committed baseline. The current
overlay contains substantial module/test extraction already. Tokei 14.0.0 code
lines measured on the implementation source, excluding comments and blanks:

| Owner file | Current code lines | Current ownership |
| --- | ---: | --- |
| `sema/src/document.rs` | 1,179 | Stage types/orchestration; nine child modules contain syntax, type inputs, body resolution, materialization, integrity, and tests |
| `pipeline/src/execution.rs` | 1,199 | Immutable DTO roots; encoding, projection, vocabulary, and tests have separate modules |
| `types/src/check.rs` | 1,061 | Checked expressions; inputs/value helpers separate |
| `types/src/solver.rs` | 1,022 | Solver orchestration; candidate engine separate |
| `sema/src/test_support.rs` | 1,950 | Test-only common helpers; family fixtures and owner tests separate |
| `modules/tests/v3_modules_v2_authority.rs` | 536 | Harness plus three thematic modules, largest 1,675 code lines |
| `pipeline/tests/v3_compiled_document_assembly.rs` | 342 | Harness plus two thematic modules, largest 1,674 code lines |
| `sema/tests/v3_document_transaction.rs` | 274 | Harness plus four thematic modules, largest 1,412 code lines |

The local size gate covers 225 Rust files, including untracked source. New
production/test files have 1,200/2,500 code-line limits; eight exact legacy
ceilings prevent growth. Test-only exemptions require guarded module mounting.
Both current-policy and MSRV Clippy run in local CI. Existing Tokei, Semgrep,
Clippy, and the policy AST parser supply the tooling; no new Rust dependency is
needed for these frontend repairs. The additional consumer CHECK regression
uses three existing workspace crates as test-only direct dependencies to build
typed pattern authority; no new engine or production dependency is introduced.
Fuzz smoke is a bounded local command, excluded from
deterministic CI and qualification.

The original structural residual is complete: `resolve_document_bodies`
now delegates typed preparation, dependency-ordered inline views and source-order
materialization to separate owners. Production function-size suppressions and
all eight legacy file ceilings were removed. Request owner/dependency repairs
were integrated in `e598a1d`, with clean source867 CI/fuzz evidence. The older
size table above describes its audit snapshot, not the current file inventory.

Avoid moving public DTO definitions into the facade solely to reduce file size:
the current facade depends on pipeline, so a reverse edge would create a cycle.
An independently required contract leaf crate would need a separate design and
compatibility review; there is no demonstrated need in these four repairs.

## Implementation and verification record

This section is a historical proof record on the explicitly named sources.
The current status is the linked 2026-10-05 session audit; older failed or
pending attempts remain unchanged.

- At the original audit, producer HEAD was
  `9060e46997dd9c62080a18bdfae06f8c162442e2` with a shared overlay.
- Frozen producer: `1b55f6233b578d1be351bb926dfbeac2a0262b70` at
  `/Users/songmin/.codex/worktrees/frontend-consumer-contract-proof/quanta-gqlang-v2`.
  Parent `c9c3f1302f53162e0cad9093f4de55a655854e75` copies the tested overlay;
  its child adds the reviewed source identity helper and test/docs. The original
  index/HEAD was preserved; no root publication occurred.
- Archived frozen consumer: `e019cb8f4e3ebb207c7ea57cd764b9059bc1083b`, formerly
  the sibling `semantica-codegraph-v2`, based on committed `8a4510aebe99` and the scoped
  repairs recorded below. This includes Why profile policy, startup hashing,
  verified-sidecar forwarding, exact captured-Python direct-return admission,
  source-issued/runtime ID projection, the PTA-test-support adapter mount,
  typed-only registered child proof emission and current test fixture APIs.
  External index is unchanged at `90732316c9281cc9ddbad45bccee5856232e25be`.
- Shared consumer main was subsequently observed at
  `3a2a9e4c1384e69b5a4ca4c8f1afd953a4096c4c`. The seven-path runtime repair patch
  was applied at `cc8ee48890ad0c6151feca7690b5f9cc2a9d26ec`, with no staging,
  commit or push by this task. Concurrent work subsequently included the fixture
  path in `2a99b7c802afec4525efd48ddb405511e47feb89` and migrated one proof-owner
  file's imports/codec API by observed `bfd47b89001f54b0053a32326dd1653de7202455`.
  Three stale fixture files already had independent current-API repairs in main
  and were excluded. Private owner proofs do not qualify this moving main.
- Producer frontend tree: `bd1e06e9ed26a70304aa4cfdc3838436b17d74a6`;
  pipeline tree: `c459d923288e04c575ea4027438465604a4ed07c`;
  vector SHA-256:
  `120414fabe70d84a3aeb60440f9a15c8fb8be2cf0246ba0290da9bc456c2ddcb`.
- Consumer canonical installed-product digest:
  `487761bd25d8130a408f682eb6889ba84a5a3cf314befa26a995d229a55859db`.
  Consumer QBC campaign closure:
  `bc57f1e2c745b5f90f9f5b94acbc63301133d482d3f4e623f5e2e5bf424bf4cc`.
  These identities have different purposes; artifact custody uses the canonical
  installed-product digest, not the campaign digest.

| Rail | Observed result | Claim boundary |
| --- | --- | --- |
| Conformance owner | PASS, 19 tests | Rename registry/preview/apply, classification, missing authority and exact unsupported shape rejection |
| Producer `just ci` | PASS, source snapshot `75a32f2e038893c2d5dbf5198a4baf28172bf8026d395266466cc95417d00eca` | 1,433/1,433 Nextest, 8 doctests, frontdoor parity, Clippy 1.95/MSRV 1.81, size/policy/architecture/external-consumer rails; local overlay proof |
| Consumer V3 topology lint tests | PASS, 19/19 | Issued-source selection/admission/execution wiring and bypass rejection; lexical policy, not execution |
| Frozen pair M23-120 | PASS on `e019cb8f4e3e` | 19/6/30 Python checks, committed producer trees/vector, locked/offline complete Cargo metadata and scoped diff hygiene; no Rust compilation; durable evidence `m23-final-source.log` |
| Source-issued runtime ID guard owners | PASS, 3 selectors, 1 test each on `939302adae21` | Original Call/issued ID preservation, foreign/missing/duplicate refusal, unissued fallback and constructor control; QBC runs `20261004T030817.130014Z-5b0038aa7276`, `20261004T031044.957088Z-e8c0dbcd210c`, `20261004T031107.953599Z-7f6f59912f96`; each 8,252 filtered |
| Captured-Python direct-return owner | PASS, 2 tests on parent `3ed3d3a9a649` | Actual retained parser/generation/source/DFG admission plus negatives; exact adapter unchanged through current consumer HEAD; QBC run `20261004T020733.249415Z-835fcfb7d8c4` |
| Typed-only Stage5 registered proof owner | PASS, 7/7 on `e019cb8f4e3e`, 4,739 filtered | Real typed carrier and issued execution session; attach then validate, missing-marker/wire-required/mismatch/unverified-output refusals and existing foreign authority checks; QBC `20261004T081304.244685Z-08ef450888d7` |
| W1/W2 final admission owner | PASS, 26/26 on `e019cb8f4e3e` | Same campaign closure `bc57f1e2…`; Graph cap/dedup and policy refusal, schema-8 golden, independent request digest, CHECK mappings and unsupported modes; QBC `20261004T081818.760298Z-8cd5c19b4103` |
| W3 final EXISTS owner | PASS, 1 executed / 35 filtered on `e019cb8f4e3e` | Same campaign closure; both independent false-conjunct cases plus positive and empty-source controls; QBC `20261004T081909.988555Z-bcefd9b0134e` |
| Optional MemSSA summary | PASS, 1 executed / 495 filtered on `e019cb8f4e3e` | Same campaign closure, exact issued/durable query keys, both profile encodings and absent MemSSA; QBC `20261004T081952.742375Z-0eaac775b4df` |
| Four-binary product build/install | PASS on `e019cb8f4e3e`; issuer status `ok` | QBC `20261004T081614.060079Z-64dd368a9e64`; exact canonical digest above, four immutable artifacts, stable clean HEAD and source; installed bundle in durable evidence `bundle-e019` |
| Installed Graph process | NOT_RUN on final `e019cb8f4e3e`; latest earlier attempt FAILED | Earlier QBC `20261004T032312.070805Z-d5c28d1e8a3a` on `939302adae21`: 0 passed / 1 failed / 25 filtered at Stage5 publication. Graph result assertions were not reached. Stage5 repair/unit proof and a rebuilt bundle are ready; they do not establish installed Graph success. |
| Consumer God source delta | PASS for the runtime-ID repair versus `3ed3d3a9a649` and the production proof repair versus `939302adae21`: no new/grown violations | Existing whole-gate baseline drift remains; no allow/baseline changes. This is not a whole God-gate PASS |
| Clean-HEAD qualification / full product closeout | NOT_RUN | Scoped clean private owner proofs and local CI are not moving-main/full release qualification |

The first shape fixture used a binding unavailable after projection and failed
in the binder. The corrected fixtures use valid literal leaf/record/list shapes,
then independently assert the semantic rejection and half-open query spans,
including Composition producer bytes 13..59. Compiler behavior was not changed
to make the regression pass.

The scoped consumer gate exposed a stale monolithic handler lint. It now checks
the mounted source-admission helper and actual issued-source worker, plan,
provider/request admission, gateway grant, and generic transport calls in their
owning function bodies, with negative wiring tests. It retains retired/bypass
guards. The gate's dirty-source and exact-pin predicates remain unchanged.

### Additional reachable CHECK mismatch

The current frontend modules contract resolves a complete authored rule path,
including dots, as one local name under the selected module. `check r.x` seeks
local `r.x` under the public frontend module. Semantica registered the issued
rule as module `r` / local `x`, then expected those backend fields on the
frontend DTO. Production and the failing fixture used this same incompatible
registration. Splitting the authored path in QGLang would change its existing
language/modules semantics.

The repair belongs to Semantica: define one explicit projection from an issued
backend pair to the public frontend module and exact authored `module.name`
spelling; register that projected catalog identity before compile and match it
back to the exact issued rule in selection/admission. Backend identity, source
membership, pattern publication, provider authority, and frontend DTO schema
stay intact. Reject unknown/foreign and colliding projected spellings. The
regression calls the real public compile helper with typed test catalog
authority; it is a compile/candidate/plan proof, not installed CHECK execution.
Runtime request-identity golden is refreshed only after its independent v4
encoder equality assertion succeeds. The final whole-lib owner run passes all
26 tests on the repaired source. Earlier runs found the stale request literal,
the production CHECK mismatch, and a test-only Scope reference registered as
Entity. The Scope fixture now reaches the existing typed unsupported-scope
plan rejection. Root/nested workspace lockfiles include only the three new
test dependency edges; no registry package version changed.

Initial admission locator: `/tmp/w1-w2-consumer-closed-owner-result.json`;
QBC run `20261003T203024.457116Z-b3ceb2a64dce` on `d2ac79f35c6`, request
`8b88be52fbe60a62088814335567f193`, command digest
`914ff6fb5b889126f29b2e3d1940c344864e976f1561df289308325d27c6c197`.
Receipts/logs remain under the fixed execution-root lane `fcp-w1-graph-proof`.
Initial EXISTS locator: `/tmp/w3-exists-frozen-owner-result.json`, QBC run
`20261003T203155.990016Z-c11c59734d96` on `d2ac79f35c6`, request
`0b1db1df56069c23d08c68bf50669af6`, command digest
`b769151227f563d2cc6a96314e2cc1d0e25b4718171fc789b682a25520b5bd09`.
The module selector uses the unique test name, executes one test, and does not
claim the other 35 cases were run.

### Installed-proof source refresh and compiler recovery

The first registered four-binary build on consumer `d2ac79f35c6` failed with
five compile errors in the pre-existing taint output refactor. Its receipt is
QBC run `20261003T203520.323233Z-80a624e309a0`; no installed process test ran.
The current committed consumer `c070e0d37fbffb30f4fe8e253329412eeddaa471`
contains the five fixes and all this task's consumer changes. The isolated
checkout now uses that exact commit; the prior commit is retained on
`codex/frontend-consumer-proof-before-baseline-refresh`. This task did not
modify taint semantics or copy a moving dirty overlay.

The refreshed canonical product source digest is
`53ed0ae1f3c24f9c9c7038c9a006e18866b19fd0570bb12c174af5d5c5a51249`.
The old owner locators above remain bound to `d2ac79f35c6`.
Refreshed admission passes 26/26 with locator
`/tmp/w1-w2-consumer-stable-frontend-owner-result.json`, QBC run
`20261003T210318.416258Z-7c42635d1026`. Refreshed EXISTS passes one executed
test with locator `/tmp/w3-exists-refreshed-owner-result.json`, QBC run
`20261003T210933.660056Z-e40f080d530a`. Both bind source closure
`7f4a475db1aa67d0de6c9d2280112937be1625538d1df0274f7c06beb77204ea`
and the unchanged owner request/command digests above.
The scoped M23-120 pin/tooling gate passed on `c070e0d37fb`.
Rustc 1.92 incremental dependency-graph assertions interrupted initial refreshed
runs in two crates. Their ignored crash dumps also changed the guarded root
directory, so QBC failed closed before producing a successful owner locator.
The task preserved lane-local incremental caches and dumps in external
`ice-quarantine-*` directories and uses QBC's documented launcher setting
`CODEGRAPH_RUSTC_THREADS=0` for subsequent proof. No source guard was loosened.
The typed `--compile-env` interface does not admit this launcher control;
the setting is supplied through its documented environment entry point.

The four-binary build subsequently exposed one wrong SDK type path in
`quanta-runtime-gateway-contract/.../leased_response_v1.rs`: the gated trait
used `response::SdkResponseV2`, while its other methods and the owning type use
`response_envelope::SdkResponseV2`. This task corrected that single path,
preserved the typed default refusal, and mirrored the hunk to the shared
checkout. Isolated commit `55e39cb977666eda0f54b0e92b71e9a27eb99438` retains
the change. It changes neither a wire type nor schema.

A later full-profile build found three further pre-existing taint compile
errors: a stale output destructure, iteration over a double reference, and an
immutable string borrow held across a vector mutation. The taint owner's
committed source already contains their corrections. The current installed
candidate is frozen at `8a4510aebe99085b1ef22fd90156281b68999920`, which
includes this task's SDK path, Graph, golden, and EXISTS changes. Prior source
tuples and owner evidence above remain historical; final source-bound owner
and installed receipts must be established on this candidate. No taint
implementation or acceptance gate was changed by this task.


The `8a4510aebe99` installed build compiled past the taint refactor but found
a paired-source API mismatch: its new frontend receipt binding calls
`ExecutionDocumentV3::matches_source_bytes_v3`, which was already present in
the original producer overlay but absent from the older frozen snapshot.
QBC run `20261003T214202.842383Z-f3b7f3e25403` failed before installation.
This task imports only that additive producer helper into the frozen closure,
reuses the canonical digest encoder, and relocates its exact UTF-8/foreign-byte
regression into the existing test child module. It grants no runtime authority
and changes no DTO fields or digest schema. The producer local CI, committed
pipeline pin, and final consumer/installed proofs must be refreshed together.


The additive helper is now committed in frozen producer `1b55f6233b57`.
Local CI passed at source snapshot
`75a32f2e038893c2d5dbf5198a4baf28172bf8026d395266466cc95417d00eca`:
1,433/1,433 Nextest, 8 doctests, all static/policy/external consumer rails;
Nextest custody run `79ee6a0858859febb3a7bb4c43fd085f` with zero skips/leaks.
The original shared helper, relocated test, and contract document match the
frozen files exactly. Consumer `df95682d407` pins the new pipeline tree;
M23-120 passes again (19/6/30 Python tests, full locked/offline metadata).
The four-binary build's canonical source digest
`a97de8612217eb85bb1867d6ccb0fb95c71e6460e70034e8dbe378029b8fa6a1`
matches an independent source-snapshot calculation. Build/installation and
final owner result rows remain pending until their actual receipts pass.


The paired-API product build on `df95682d407` (QBC run
`20261003T220533.053415Z-4da4b122db1f`) compiled past the prior API errors,
then failed with `E0275` while proving `Send` for the SQL worker's existing
nested query-dispatch future. Rust's default compiler recursion limit is 128;
the existing runtime crate already uses 256. This task adds that same finite
compile-depth attribute to `quanta-sql-rulepack-worker/src/lib.rs` only, preserves
`forbid(unsafe_code)`, and mirrors it to the original shared checkout.
Normal commit formatting/safety hooks passed. Isolated consumer commit
`5f5216d35d3f1b9ab6e514e19674f12f1e04a8eb` records the three-line change.
It changes no runtime recursion, work budget, DTO, or effect policy.
A fresh registered product build is required before installing or issuing
any successful process proof; the previous canonical digest remains historical.

### Remaining full-profile compilation repairs

The frozen consumer imports two exact SDK owner fixes from committed
`303a896f6098`: test-only imports are guarded, and the private JSON wrapper
remains available to its production Preview/Rollback/Audit callers. Legacy
Apply refusal and the issued V3 route remain unchanged (`9256f72111dc`).

The following small integration repairs then allow the existing full profile
to compile. They are mirrored as owned hunks to the original consumer:

- Bootstrap declares the absent source/sink authority issuer as `None`, matching
  its absent external module authority. A valid manifest without MemSSA projects
  `ScalarV1::Absent` for that role (`ef0681ec5852`).
- Resident startup imports the existing five ingress types/traits. LSP dispatch
  flattens its nested `Result` with `?`. The ingress facade re-exports the existing
  leased response type for the shared adapter; no production dependency is added.
- Source text lookup iterates the borrowed view's existing `(key, text)` iterator,
  preserving charging before cloning. Graph witness size uses an `Option` fold,
  preserving checked overflow and the typed budget refusal.
- Fast call graph builder forwards the existing call graph control to its inner
  builder, preserving the request's deadline/cancellation propagation
  (`31924eec3e52`).
- The optional MemSSA regression encodes both valid manifest profiles and checks
  exact IFDS/PTA values and absent MemSSA. Its paired fixture includes the required
  profile wire tag (`f397f105379`). Independent static review found no remaining
  fixture/decoder mismatch; runtime owner proof is still required.

The failed product builds retain immutable QBC receipts. These edits do not
establish successful compilation or process proof until a new source-bound
receipt passes. Compiler warnings outside the affected owners are not treated
as new defects or silently suppressed.

### Graph proof compile isolation

The first installed Graph owner attempt on `f397f105379` failed while compiling
an unrelated dense SAMPLE test in the same integration target. No Graph
assertion executed. The scoped Graph test uses the ordinary full daemon and
existing promoted Rulepack fixture; it needs neither the Pattern publication
fixture nor dense SAMPLE's publication helper.

Consumer `2e4d3100e01` adds `gqlang-graph-process-proof = ["dist-full"]` and
registered class `quanta-cli-server.no-default.3c0734102bd7` with features
`["dist-full", "gqlang-graph-process-proof"]`. Only the Graph workspace helper
and existing Graph test gain an alternate cfg. Its complete body, expected
path IDs, empty result, budget/foreign-source refusals and installed artifact
checks remain byte-for-byte unchanged. The old full feature/class and both
dense callers are unchanged; the Graph test still participates in the full
feature. This is additional scoped proof, not a replacement for that full gate.
Independent static review and the existing feature-isolation snapshot tests
passed. Product QBC run `20261003T233709.648755Z-c0f666025f59` passed with four
immutable binaries and the independently checked canonical source digest
`e43abd5bf5d4013c6f7319025c181980cc4910eab9675101ccc5615fb8e79b08`.
Locator: `/tmp/frontend-consumer-2e4-product-build-result.json`.
The first isolated compile found a common relation assertion incorrectly
gated behind the full provider feature although unconditional tests call it.
Consumer `1f459be3615` removes that one test-helper cfg; its checks and callers
remain unchanged. The renewed four-binary build passes at QBC run
`20261003T235011.763736Z-874eeb04b6eb`, canonical source digest
`ef37464409a4d86feebfb3076de9e7cf5115459390ede6e70cc991ae50087132`, locator
`/tmp/frontend-consumer-1f4-product-build-result.json`.
The MemSSA regression target exposed five stale test fixture compile errors.
Consumer `914a4811b2b` fixes the retired classifier name, initializes both
creating actor fixtures with the production constructor's unbound `None`
attach generation, and uses existing diagnostic-owned callsite/evidence batch
constructors. Assertions and production behavior are unchanged. Normal commit
formatting/safety checks pass. The MemSSA owner then passes one test with 494
filtered, QBC run `20261004T000503.349695Z-15a8c09df020`, locator
`/tmp/frontend-consumer-optional-memssa-914-owner-result.json`.
The isolated installed Graph owner result remains pending on this final source.

## Final action order

1. Finish frozen-pair W1 admission, W2 golden, and W3 result owner proof, with
   nonzero counts and exact source-bound QBC receipts. Share a captured source
   snapshot where the registered feature closures permit it.
2. Build/install the existing four-binary immutable product recipe and run the
   registered unique-name Graph process owner selector on the same source/lane/artifact
   tuple. This specific Graph test uses its promoted Rulepack fixture and does
   not require the SDK wheel or signed model policy needed by the full release
   front door. Preserve installed-path/digest/receipt/target-root checks.
3. Integrate the reviewed producer closure into the original shared checkout
   through a serialized owned commit after source/ownership reconciliation;
   keep consumer pins equal to its committed trees. Preserve concurrent work.
4. Next structural batch: decompose `resolve_document_bodies` by family and
   lower its existing function-size exception. Only split other legacy files
   when their owning responsibility is touched; keep the exact no-growth gate.
5. Run bounded local fuzz smoke when parser/formatter changes; retain corpus,
   crash artifacts, source/toolchain identity. No hosted workflow or scheduled
   service is required for the solo local workflow.
6. Treat S21/full product matrix and the stale legacy joint receipt target as
   separate qualification work. Repair proof producers at their owners before
   making a full-closeout claim; do not add an optimizer/SMT engine to satisfy
   these source/contract repairs.

### Installed daemon startup profile repair

The Graph integration target compiled on `914a4811b2b`, but its first installed
run selected system Python 3.9 for source identity checks; the existing Python
3.12.12 environment resolves that tool environment failure without a source
change. The next run reached installed daemon startup, where the unchanged
profile capability validator rejected `analysis.why`. That supported public
surface advertised Instant/Interactive/Deep although its daemon-native
`WhyEffectExecutionServiceV1` never consumes a profile and has no corresponding
executable runtime profile capability. Original Semantica `8e988a9a527e` has
the same inconsistency.

Frozen consumer `e0bbb98bdd3ef6437b6a17369f7ba8ef97fa6c55` makes Why
`SurfaceFixed`. Default Why execution remains available; explicitly supplied
profiles now receive typed Validation `profile_not_supported_for_surface_fixed`
before source bootstrap. This is a public CLI/doctor policy behavior change;
it does not alter the GQLang DTO schema or create a runtime capability.
The startup validator and Graph process assertions remain unchanged. Contract
and ingress regressions cover default acceptance and explicit refusal with and
without a file target. All other 12 explicit profile policies have existing
capability bindings. Owned changes were mirrored to the original shared
consumer; no original index/commit operation was performed.

Why owner proof: three tests passed on this frozen source (493 filtered), QBC
run `20261004T002341.973300Z-3b5fdbf89689`, source closure
`bffab0c487924aa7221849cc03bc7ae237118912f1518410077cc2fd822b7a64`.
Final source-matching build/installed Graph execution remain pending.

Why contract owner: 1 passed / 178 filtered, QBC run `20261004T002700.921350Z-19e80c930fed`. Public profile registry: 3 passed / 493 filtered, QBC run `20261004T002739.049896Z-444f9b74ca57`. Both bind source closure `bffab0c487924aa7221849cc03bc7ae237118912f1518410077cc2fd822b7a64`. The registry still rejects a missing executable profile; the validator was not relaxed.

The registered product rebuild passes on `e0bbb98bdd3e`: four immutable artifacts, QBC run `20261004T002829.940187Z-b4eca9ab9ae7`, independently checked canonical source `32436bdbb5b7363256234722269e5f2d8a39a1493019fea4f0493cdfd1b1d432`. Final bundle issuance and installed Graph assertions are still pending at this checkpoint.

### Installed startup cost and narrow profile repair

Two source-matching installed Graph attempts on `e0bbb98bdd3e` passed
compilation but failed daemon readiness at the unchanged 60-second deadline.
The diagnostic run (`20261004T003822.375659Z-746d4369cf64`) retained a failed
QBC receipt. Graph fixture/attach requests had not executed. The sampled daemon
used CPU and held the installed 837,868,768-byte server executable open; the
stripped sample has no function names, so it cannot independently identify the
exact innermost phase. Current source performs whole-executable SHA verification
for aggregate local identity, then Darwin scan-worker custody copies/fsyncs and
verifies the same executable before the listener. This is evidence of a large
startup cost; a specific phase bottleneck remains inference pending phase logs.

`6f0dae444367d0d882b13de1447e63d2a9778171` keeps the existing SHA-256
implementation and adds only `profile.release-full.package.sha2.opt-level=3`.
The local product profile otherwise inherits dev/opt-level=0. The pinned ARM64
sha2 0.10.9 software compressor is non-generic, so the package override optimizes
that repeated computation without a new crate, version, feature, algorithm or
custody bypass. Copy/fsync costs and the 60-second deadline remain. Effectiveness
is not yet claimed. DEBUG startup phase IDs and cumulative milliseconds now
make bootstrap, profile validation, scan pin, adapter and ingress timing visible
without printing paths, digests, authority or source payloads. Normal private
commit hooks passed; owned hunks were mirrored without touching the original
index. A new registered build/install and process result are required.

### Successful readiness and verified-sidecar forwarding

On `6f0dae444367`, the installed Graph regression passes daemon readiness but
fails the actual workspace attach before Graph query execution. DEBUG phases
record bootstrap=1,556 ms, scan pin finishes at 4,048 ms, adapter at 4,057 ms and
ingress at 4,058 ms. The preceding unoptimized builds timed out at 60 seconds;
these are single local observations, not a general latency benchmark. The
60-second deadline and all custody checks remain.

Attach returns typed Bootstrap `daemon_search.attach_failed_v1`: the initial
aggregate provider lacks a concrete verified host sidecar. Three independent
source audits agree the server supplies the verified installation packet but
`DaemonSharedExecutionAuthorityV1::from_validated_bootstrap_v1` discards it when
building the client. The sibling SDK client constructor already installs that
packet. This is an authority forwarding defect, not a missing runtime engine or
a need to invent host/root authority.

Frozen consumer `62cf18e4a9256b20d82e70d12b7bae3dc0967a95` retains the
original verified packet before bootstrap field moves and installs it through
the existing client method. Installation failure maps to typed
`RuntimeInitializationFailed`; provider identity validation and missing-sidecar
refusal remain. The common search-only constructor retains its existing
workflow-factory behavior under feature unification. Normal private hooks pass;
the ten-line owned repair is mirrored to the original consumer. The unchanged
installed Graph attach/path/reachability regression is the independent positive
proof, still pending on this new source.


### Source-issued solver ID projection repair

The installed Graph run on consumer `3ed3d3a9a649` reached real workspace
publication and then rejected a valid direct return call with
`pta_direct_static_source_issued_call_unavailable_v2` (QBC run
`20261004T021645.865581Z-6ce877cd1050`, zero passed, one failed, 25 filtered).
The exact source/DFG join retains producer solver IDs derived from raw path/FQN
BLAKE3 bytes, while the runtime physical owner uses canonical-CBOR IDs. The
fallback incorrectly required those two representations to be equal.

Consumer `273cf4f0e54f` projects the already source-admitted Call to the selected
runtime owner using the existing producer canonical ID helper. It retains the
original Call and complete issued arguments, requires one source Call at the
site, exact path/FQN, target validation and return identity, and keeps the
constructor physical-ID guard. No solver identity is rebound and no authority
is fabricated. Work is linear in path/FQN bytes, without a per-call function
map or allocation. The focused regression distinguishes source and physical
IDs and rejects another path/FQN, return, missing issuer and mixed duplicates.

Private child `edf6b9bb623` repairs the new test module mount. Private child
`939302adae217a413ab66d49e5bd7630d600ee60` exposes the existing materialization
port adapter in the PTA-test-support/no-taint feature closure; the implementation
and memory reservation behavior are shared with the real taint output. The
first lib-test attempt exposed this feature closure mismatch, rather than a
retired test constructor. Actual owner and installed proof results must be
recorded before calling this repair complete.

### Isolated source custody recovery

Codex desktop auto cleanup removed the unregistered consumer checkout twice.
Git worktree locking retained its admin entry and exact HEAD but did not protect
source files. Desktop logs identify `worktree_auto_cleanup` and the exact
source path; QBC's simultaneous orphan sweep only removed cache namespaces.
The worktree was restored from the retained private commit at the same path.
The pending 12-line feature repair was recovered and committed. Immutable
protection during proof covers included source entries and the Git link, with
QBC's excluded output directories precreated and writable. Afterwards, remove
that protection and retain the private consumer/index checkouts outside the
app auto-cleanup directory. Failed/partial attempts remain historical evidence;
none may be promoted to a passing behavioral claim.


### Typed-only registered child proof emission

The installed Graph attempt on `939302adae21` gets past native direct-return PTA
admission and fails at `product.taint.stage5_custody` with
`TAINT_EVIDENCE_STAGE5_EXECUTION_PROOF_UNAVAILABLE`. The current process test
uses the signed repository-local promoted Rulepack fixture; this error is not
evidence of missing Rulepack authority. It occurs before Graph result execution.

Two independent audits trace the same cause. Product Taint sets the existing
runtime-persist-disabled marker. Child dispatch may then omit outward byte
transport while retaining the verified typed carrier's admitted canonical
output. Kernel child-output validation explicitly permits that mode. Product
`registered_child_proof_v1` instead requires nonempty outward bytes, so it
returns the otherwise eligible fresh child without a registered proof. The
semantic helper's existing no-memo retry cannot satisfy this outward-byte guard.

The owning repair must bind the proof to nonempty admitted typed canonical
bytes, and allow empty outward bytes only in the existing permitted typed-only
mode. If outward bytes are supplied they must still equal the admitted bytes.
Keep fresh execution, Product self-port, internal dispatch, exact request,
source generation, session, scope, snapshot and output binding. Do not clear the
no-persist marker, fabricate a proof or relax the Stage5 validator. Focused
positive and missing-marker/wire/mismatch negatives plus the unchanged installed
Graph test are required before closure. Root cause is static caller-to-owner
tracing until the actual post-repair execution passes.

Final owner result: private `d7c54bdcd4da` implements the admitted typed-byte
selection and proof hashing. It preserves the source/session/scope checks and
allows no-wire output only when the existing runtime policy permits it. Supplied
outward bytes must still equal admitted canonical bytes. The hash is computed
after the eligible source-authority checks, avoiding new hashing work on
unsupported internal child shapes. Independent source review found no authority
relaxation; the production AST size delta introduced no new/grown violation.

The first focused runs exposed stale API fixtures, then a test lease without an
issuer, then an insufficient fixture allocation budget. Private `e0164531c079`
updates only the three stale fixture files; `d0c25dcf941` uses the existing real
logical-byte issuer; final `e019cb8f4e3e` changes that test's budget to 16 KiB.
No production budget, admission gate or assertion was weakened. Final owner run
`20261004T081304.244685Z-08ef450888d7` passes 7/7. This establishes the scoped
proof-emission owner behavior, not installed Graph execution.

### Final handoff and resumption boundary

- Final clean producer/consumer/index heads are the tuple in the current proof
  table. Shared producer remains dirty at `9060e46997dd`; shared consumer main
  moved to `3a2a9e4c1384`. Never label either main qualified by the private tuple.
- Durable evidence directory:
  `/Users/songmin/Documents/code-new/.frontend-contract-proof-evidence-20261004`.
  It contains final owner/build result locators, the exact installed bundle,
  final scoped pin-gate log, immutable QBC receipts and non-JSON logs, and both
  the full historical repair patch and the seven-path current-main patch.
- `run-installed-graph.py` checks exact clean HEAD, installed digest, four role
  SHA-256 values and build receipt/target identity before the unchanged test.
  It has not been executed on `e019cb8f4e3e`; no final installed-success locator
  exists. Re-read the retained source and artifact identities before resuming.
- The persistent execution goal reached `usageLimited`. Already-running owner
  tests finished successfully; a new installed Graph run was not launched.
  The goal is incomplete. No full release, S21, new fuzz run, clean-main receipt
  or current-main installed result is claimed.
- Preserve unique private commits and ignored installed evidence before source
  cleanup. Remove the temporary immutable source protection only after archive
  verification and when no runner owns the source. Keep the producer sibling.
  Verified incremental Git bundles are retained as
  `quanta-gqlang-v2-1b55f623.bundle` (prerequisite `9060e46997dd`) and
  `semantica-codegraph-v2-e019cb8f.bundle` (prerequisite `8a4510aebe99`) in the
  durable evidence directory. They preserve the unique private commits and
  require the original repository's prerequisite objects; they are not
  standalone repository backups. Installed bundle and copied receipts are
  retained separately. The subsequent scoped retirement is recorded below.
- Compatibility review must include the earlier Why policy change: default
  Why requests use the existing fixed surface; explicitly authored
  Instant/Interactive/Deep profiles now return typed validation refusal.
  Its owner tests pass; this is a behavior change requiring review during main
  integration, not a transparent module extraction.

### Scoped consumer retirement and next main proof preparation

The consumer checkout has been retired after all of its execution processes
ended. Command-line and open-file checks found no remaining physical checkout
holder. Only
`/Users/songmin/.codex/worktrees/frontend-consumer-contract-proof/semantica-codegraph-v2`
was removed with `git worktree remove`, after clearing that checkout's temporary
immutable flags and unlocking its own Git worktree entry. The managed parent,
producer sibling `1b55f623`, index sibling `90732316`, and shared GQLang
documents/dirty changes remain.

External conservation and verification:

- `semantica-source-e019.tar.gz`: complete 31,306-file tracked source archive.
  Independent Git blob/tree reconstruction equals original tree
  `f977ce43195fecfdd8e2459e44e9e00f7319cb24`.
- `casectl-nine-files-e019.tar.gz`: all nine tracked/excluded casectl data files,
  including its SQLite evidence store and attempt ledger. Byte roundtrip and
  original Git blob checks passed; these records are historical source data.
- `consumer-ignored-six-files-e019.tar.gz`: all six ignored bytecode files,
  also checked by byte roundtrip.
- Incremental Git bundle and `fixture-d0-e019.mbox`: private commit metadata and
  both actual fixture commits retained. Bundle prerequisite verification passed
  in the original repository.
- `consumer-archive-verification.json` and `consumer-retirement-record.json`
  bind the archive checks, removed source and preserved siblings. The previous
  installed bundle and all retained proof receipts remain outside app cleanup.

Main integration includes the real fixture issuer and 16 KiB fixture budget;
current main previously lacked the typed-only fixture, so both require the
reviewed seven-path prerequisite repair. Six owner files remain byte-identical
to the private proof source; the registered child proof production file has a
subsequent dispatch-port/codec ownership migration. No current-main Rust or
installed Graph result is claimed.

`next-main-graph-proof-inputs.json` contains the successfully resolved current-main
Graph owner request, observed source roots, fresh build recipe, artifact
requirements and the unchanged result/rejection oracle. Actual fresh source
capture refused concurrent mutation of
`docs/plans/sep-20-runtime-compile-link-dx/evidence/dx-current-source-audit-2026-10-04.json`.
Its status is `awaiting_stable_source_snapshot`, not a frozen executable input.
After writers settle, re-freeze current main (or a new exact snapshot from that
main), re-resolve affected owner requests, build and issue a fresh four-role
bundle, then execute Graph. The archived `bundle-e019` and old build receipt
cannot qualify the new main. This preparation starts no Rust build/test and does
not resume the usage-limited persistent goal.
