# Oct-10 bugbash implementation closeout

This records the implementation outcome for the six active findings in [FINAL-TRIAGE.md](FINAL-TRIAGE.md). The triage document preserves its original source snapshot and decisions.

| Bundle | Current result | Evidence |
| --- | --- | --- |
| A / M04-01, M04-02, MF-01, MF-02 | Modules owner changes and regressions implemented in the dirty QGLang checkout. Combined local `just ci` passed: Nextest 1,531/1,531, skip/leak 0. At 4,096 edges, topology owner median 39.007→17.251 ms and family median 36.462→19.607 ms. Certified reachability showed no clear latency change; large-visible M04-01 isolation was not measured. | [IMPLEMENTATION-A.md](IMPLEMENTATION-A.md) |
| B / CG-01 | Semgrep includes execution-contract source and requires it in the scanned-path inventory. Real violation and missing-path fixtures pass; production findings 0. Included in the combined `just ci`. | [IMPLEMENTATION-B.md](IMPLEMENTATION-B.md) |
| C / DP-01 | Semantica lock drift was isolated and the lock update is in Semantica `cab618d`; current observed Semantica HEAD `44371bae` passes full `cargo metadata --locked --offline`. An independent E0583 module-path correction is included in `44371bae`. The V3 consumer vector remains unexecuted: QBC admission rejected moving sibling overlays; raw compilation reached 15 errors in `quanta-runtime-source-authority-wire`. | [IMPLEMENTATION-C.md](IMPLEMENTATION-C.md) |

## Verification boundary

- QGLang combined local CI source snapshot: `47e3fcf41a4f54f5704ecb1bb8fce35c2261dffc6d679bfae4e8c30c8cb7e9e8`. The implementation source and policy files were not edited after this receipt; later edits were to evidence documents.
- QGLang implementation remains uncommitted on HEAD `25fad76cf5b4510244157c2e009b4dc37aa958ff`. Other writers' dirty documentation exists in the shared checkout. `just qualify-local` requires a clean committed HEAD and was not run. Hosted CI, installed product, and runtime behavior were not verified.
- The Semantica source-authority-wire failure requires a coupled owner/caller migration to the current output-slot and birth-slot APIs. A `String`-return alias would evade that custody contract. Until that owner change and a stable sibling snapshot are available, consumer compile/vector and QBC qualification remain open. Do not infer consumer success from the lock metadata or QGLang CI.

No commit or push was performed by this implementation task.
