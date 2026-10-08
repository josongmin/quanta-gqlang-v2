# Closeout root-cause analysis — R77

Updated: 2026-10-08. This records the R77 cause and dependency map for the
[18 active tickets](tickets.md). [Remaining work](../../work/remaining.md)
owns their states. All execution is local.

Successor repairs and actual runs are recorded in [R79–R81 execution](execution-r79.md).
The hierarchy mounts are integrated in `d29ac508b4d`; live Java member handoff
now borrows the parser's authenticated FileId internally and no longer needs the
external `members_v3` argument. Preserve the R77 observations below as history.
Its `--no-deps` metadata result did not prove full locked dependency resolution;
the later complete resolution and two reached pretest failures have separate inputs.

## Inputs and evidence limits

- Documentation input: clean QGLang `b8dd3c29e43a`.
- Committed Semantica input: `f574fa82bf123c817e80fc3afc795b4dd033ea3f`.
  Fixed proof siblings are QGLang `e9a268145ca3`, Index `f60609fe2d28` and
  QMP `8ea774894335`; source digest
  `8c32be4f140bab2bea0c25ce4e1274a7cfe3c835a71b3c42998fb20599ed308f`.
- Live Semantica readback starts with 2,723 dirty/untracked entries and no
  integration barrier. The evidence records hashes of 54 inspected files and
  the overlapping claims. This is a per-file static observation, not an atomic
  source snapshot or a compilation verdict for current WIP.
  The closing readback has 2,742 entries; one inspected Java parser file changes
  during the audit. The recorded member/publication/facts seams retain their
  sampled hashes. Changed parser bytes are not promoted to a compile verdict.
- Local evidence root:
  `/Users/songmin/.codex/task-evidence/oct8-substrate-rca-r77/`.
  `parser-diagnostic-breakdown.json`, `module-mount-readback.json`,
  `receipt-readback.json`, `metadata-observation.json` and
  `source-readback-{before,after}.json` distinguish these inputs.

## Confirmed causes

### 1. The committed parser packet is incomplete

R72 parser all-features lib check and R75 PTA adapter SCC request reach the same
105 parser diagnostics on the fixed tuple. Both execute zero tests. They are
one shared compile prerequisite, not two failing SCC algorithms.

| Diagnostic class | Count | Cause and owner action |
| --- | ---: | --- |
| `E0583` module lookup | 11 | Parent files mount bodies at paths absent from the committed tree. Integrate the existing bodies with their exact mounts |
| Missing names/types/traits and private module access | 32 | `E0425/0433/0412/0405/0422/0603`; align caller imports, exports and feature guards with the canonical owners |
| Duplicate definition, lifetime, associated type and missing trait item | 4 | `E0428/0106/0220/0046`; complete the declarations and implementations; some diagnostics may cascade from missing traits |
| Denied `unused_imports` | 58 | Restore the intended reachable code/feature closure, then remove or scope truly unused imports; retain the lint |
| Total | 105 | The count is a diagnostic inventory, not 105 independent repair tickets |

The inference from these diagnostics and existing WIP is an incomplete coupled
module/API integration. A fresh frozen build must determine which repairs remain.
Source inspection alone cannot certify that the 105 errors are fixed.

Current readback finds explicit corrected mounts for eight of the eleven modules.
Three bodies remain under `java_workspace_resolution_syntax_v1/hierarchy_authority_v1/`
while their parent, itself mounted with `#[path]`, still uses bare declarations:
`correspondence_v3`, `edges_v3`, `implicit_object_v3`. The historical compiler
searched their parent's directory instead. Review those exact mount pairs in
FCP-COMPILE and prove them with the real parser owner; this is a static remaining
mount risk, not a new live compiler result.

The committed Java generated-callable candidate separately imports
`OriginalSemanticSharedBackingV3`, absent from committed Foundation's definition
and exports. Its supplier and the new DFG evidence backing exist in WIP.
The supplier closure is tree-sitter → Kernel → Foundation → SourceWire → parser
and DFG consumers. Preserve the existing shared holder and paid birth/retain
contracts; validate and integrate the complete compatible packet.

### 2. The latest input cannot yet be captured for owner proof

The reached Foundation feature-alias error is repaired in WIP:
`semantic-original-admission` now forwards through the dependency key
`quanta_runtime_query_floor`. R77 offline locked all-features Cargo metadata
succeeds with 447 workspace members; the Foundation manifest and root lock hashes
stay unchanged during this check. R76's 327-package observation retains its older
input. Metadata success does not establish compilation.

Subsequent owner attempts abort while files or external sibling overlays change
during hashing/materialization. The latest contract-core attempt identifies
`quanta-runtime-ingress-core/src/ingress_spec.rs`. No owner run locator is published.
This is source-capture rejection before compilation, separate from the fixed
tuple's parser errors. The guard is rejecting mixed inputs as designed.

Action: reconcile overlapping writers, choose the complete compatible postimages,
capture Semantica and all path-dependency siblings under source custody, and hold
the snapshot execution lease through the run. Reuse the normal shared-main
coordinator for integration. Repeating tests on mutable main or bypassing the
guard cannot discharge a ticket. This prerequisite belongs to FCP-COMPILE's
execution plan and does not create a new implementation ticket.

### 3. Original output success still has three source obligations

| Boundary | Current source evidence | Required change |
| --- | --- | --- |
| Runtime/SDK → PTA | `original_java_workspace_v1.rs::produce_live_native_workspace_generation_v1` requires five arguments, including genuine `members_v3`; runtime-analysis production and SDK fixture calls still pass four | Carry actual source/`AnalysisFileIngressV1` members through both callers and preserve source/operation identity |
| PTA and retained DFG publication | PTA `issue_workspace_transactions_v1` prepares the workspace but deliberately returns `UnsupportedOperation`; retained DFG refuses ordinary source-world/digest-map births under Original custody | Complete controlled extraction/transaction output and source-world/publication births under the same operation/group; retain temporary/final permits until their actual last owner |
| General adapter → context → extractor → facts | `FactExtractionRequestV1` and `DefUseFactsParseContextV1::single_parse` carry erased Native current, without canonical output admission | Thread the existing admission through the real producer and final facts keeper; admit before growth and preserve ordinary unsealed compatibility |

These are source-confirmed unfinished boundaries. The added member argument alone
cannot close the two deliberate publication refusals or the general facts seam.
The existing runtime → PTA → DFG issuer caller is present. No second issuer,
allocation owner or replacement language migration is required.

### 4. Most other open IDs are acceptance gaps

SCC core20 and graph-utils facade1 are GREEN on the fixed tuple, with verified
publication hashes. Abstract worklist77 and the earlier ordinary-language/SCV2/D5
612 retain their own accepted input scopes. They do not qualify current WIP or
other consumers. Core/facade source and acceptance are not reassigned.

Shape/Cost SDK feature closure, PTA/Andersen/runtime-analysis, taint's explicit
ignored 50k case, DFG promotion, remaining SCC callers, Native Preview, real SDK
deadline tests and product projection/policy owners still need their actual
results. Earlier RED or missing receipts do not by themselves prove a new runtime
bug. Open a source repair only at a reproduced responsible owner.

SDK deadline2, community empty/cycle2 and the parser no-Java export correction are
authored integration cuts. Qualify their complete dependencies and integrate the
owned postimages. C/Go/TypeScript local-budget dedup and Kotlin exact binding are
already integrated; their remaining obligation is Original control/output proof.

### 5. Installed CHECK is a separate release prerequisite

Actual Graph/CHECK process proof needs receipt-bound binaries, a compatible host
and cleanup/output oracles. Positive installed CHECK additionally needs the
release-owned signed x86_64 Linux profile, immutable root-owned slots, independent
approval/key and active generation. No current installed result is available.
The auditor host or missing local environment keys do not prove that another
runner lacks these assets. Provision and verify them through FCP-INSTALL; reuse
the same process proof for SUBSTRATE-PRODUCT. Hosted CI is outside the queue.

## Final execution order

| Order | Existing tickets | Concrete exit condition |
| --- | --- | --- |
| 1 / P0 | FCP-COMPILE | Reconcile and capture the complete supplier/module/caller tuple; metadata, parser no-Java and Java, actual SDK/runtime owners compile; integrate the compatible packet through the normal coordinator |
| 2 / P0 | CHECKED-RETAINED-CONTROL | Genuine members reach the existing producer; PTA transactions, DFG publication and general facts output have admitted success plus typed rejection and final-drop lifetime proof |
| 3 / P0–P1 | CHECKED-DEFUSE, CHECKED-LANG-SUPPLIERS, CHECKED-CALLBACK-FIRST-CAUSE, CHECKED-CURSOR | Real owner facts/order/binding and Original budget/work/cancel/first-cause controls pass, including the five original-issued traversal witnesses |
| 4 / P1 | D1-PROOF, D2-PTA-PROOF, D2-TAINT-PROOF, D2-CALLER-COVERAGE, D3-PROOF, FCP-PTA | Actual consumer results pass on their frozen closure; select ignored depth explicitly; verify historical bootstrap refusal at its real owner; reuse accepted core/facade/worklist scopes |
| 5 / P1 | F2, F7 | Native minimal/full registry→worker→Preview/provider-off and real SDK deadline/domain results pass; integrate authored SDK2/community2 after their owner proof |
| 6 / P1 | SUBSTRATE-POLICY | Affected feature/topology/Honesty/GOD/FFI/input/generated-source gates pass on the integrated tuple, without lowering gates or reapproving a global baseline |
| 7 / P1 | FCP-PROCESS, FCP-INSTALL, SUBSTRATE-PRODUCT | Receipt-bound Graph/CHECK process and installed acceptance complete; roll up the same evidence for F1/D6/D7 |

Orders 3–5 can use independent existing owner closures once their actual
dependencies are complete; they need not all wait for Original output success.
Compile/source capture is the first blocker for the affected broad owners.
FCP-INSTALL is required for full installed-product completion, not for pure SCC
or frontend acceptance. Do not calculate a completion percentage from mixed
source, test and installation states.
