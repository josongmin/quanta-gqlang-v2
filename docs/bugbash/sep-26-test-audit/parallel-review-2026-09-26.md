# Sep 26 parallel test audit and serial integration

Initial base HEAD: `78f477056cfbc726441d298fe243d3d34efb566f`. A concurrent task committed the integrated fixes as `a1776b70b7e3dd1174b76c5db01fcb08da206c1a`; the final gate freezes that HEAD plus this task's remaining test/document delta. This task does not issue a commit. This review uses the shared dirty checkout. Historical packets and receipts are not implementation authority. Three read-only reviewer lanes covered syntax/HIR/binder, types/modules/sema, and front doors/tooling; follow-up lanes covered algebra, logic and planning. The integrator owned production mutations, source restoration, and final verification.

## Confirmed code fixes

| ID | Failure and correction | Independent regression rail |
|---|---|---|
| PC-01 | A Join exit left its children's cardinality values on the shared post-order stack. With joins on both sides of EXISTS, the SemiJoin read a retained right-side child as its left result and reported `LinearInInput` instead of `Multiplicative`. Each exit frame now records its own value base, reads only its ordered child results, then truncates them before publishing one result. The iterator view avoids slice-index panics and per-node allocations. | Public source compile matrix: left scan/join × right scan/join × EXISTS/NOT EXISTS, eight valid cases. The original committed fold compiles but fails this oracle; the corrected fold passes. |
| PC-02 | The CLI parity decoder accepted incomplete `\x1` as a valid hex escape, allowing an invalid line representation to compare equal to a stdio character. It now requires exactly two digits for `\x`. | Literal valid `\x00`, `\x01`, `\x0f`, and malformed/truncated controls; decoder guard removal fails the Python test. The CLI encoder independently pins `U+0001` to `\x01`. |

PC-03 is test tooling only: the structural witness recorded a call name but neither arity nor an exit boundary, so `f(g(a), b)` and `f(g(a, b))` produced identical preorder witnesses. Call markers now record arity. A new independently authored grouping inequality fails the original helper, passes the corrected helper, and still treats transparent parentheses as equal.

PC-01 changes qualitative plan metadata, not downstream runtime execution. Its frame fix and algebra owner-order oracle were contributed by the concurrent optimization-audit task, reviewed here and integrated with the public source matrix. This task completed the allocation-free iterator variant and final integration. Unrelated prompt-manager, corpus and formatter work is preserved.

## Test improvements

| ID | Contract now pinned |
|---|---|
| TC-01 | Analysis lookup returns the exact item for each of three different source-ordered dispositions; absent identity still misses. |
| TC-02 | Nested host calls with matching Antitone effects retain the complete authored effects tuple at the parent. |
| TC-03 | Sixteen authored collection carriers admit width two and reject width one at the second element's exact span. Project/group/order separately enforce projection width. Expression-node accounting remains cumulative across two document roots: four nodes admit, three reject. |
| TC-04 | Public authority inputs retain all thirteen scalar types for required/optional relation fields and Rule parameters, including exact names/order. A compiled optional relation field reaches the execution schema with its Optional wrapper. |
| TC-05 | Public source projection and predicate uses preserve false payloads. A separate raw Bool HIR owner rail pins both false and true; source predicate syntax alone does not select that raw-HIR conversion branch. |
| TC-06 | Stdio sessions preserve the complete literal response through three-byte short writes and an initial Interrupted result; zero writes return WriteZero without flushing a fabricated response. |
| TC-07 | Residual filter conjuncts retain authored left-to-right identity in algebra and the literal comparison order `>`, `!=` in the public execution table. |
| TC-08 | Same-primary diagnostics retain stage, severity, code and message tie-breaks under independently authored permutations. |
| TC-09 | Nullable mixed Asc/Desc source sort retains the two authored directions, Last NULL placement, Optional Int key type and resolved field identity in the final execution schema. |
| TC-10 | A Flow composition producer and Taint body retain exact graph kinds and complete ordered role/target pairs, including Sanitizer, Scope, Window and Frontier. Flow has no entity-target capability, so the body deliberately uses admitted host endpoints rather than fabricating a Flow-binding target. |
| TC-11 | Export/Explain/FIND/CHECK source items retain independently expected Execution effect, evidence and policy-family tuples and exact authored item identity. V2 plan/accessor type checks are not used as V3 value expectations. |
| TC-12 | Private expression projectors retain quantified binder identities/body, EXISTS polarity/nested relation/correlation, and local/host callable target and argument keys. These are owner projector tests: rule-only quantifiers/calls issue no current Execution item, and source EXISTS lowers into SemiJoin/AntiJoin rather than retaining an expression DTO root. |
| TC-13 | Structural witness distinguishes nested call argument regrouping, including a zero-argument nested call, while preserving parenthesis transparency. |
| TC-14 | The 100,000-deep raw shape formatter compares the complete independently constructed expected string, including its literal leaf. Its bounded child watchdog and exactly-once post-assertion completion protocol remain enforced. |

Current production behavior was already correct for TC-01–TC-06 and TC-08–TC-12; these are regression coverage improvements, not additional observed production bugs.

## Executed evidence

`target/sep26-parallel-complete/accepted-mutation-results.json` records twenty-seven compiling Rust regression injections, two compiling test-oracle negative controls, and one Python guard removal. Every accepted run executes a nonempty test selection, fails an assertion, and restores the original source bytes. Compiler failures and zero-selected runs are not accepted as kills. The two test-oracle controls remove Call arity again and replace only the deep shape fixture leaf `1` with `0`; the latter survived the old length/prefix/suffix check and now fails the complete equality. These are not two additional production defects.

Before improvements, eight compiling injections survived the selected full owner/consumer suites (`before-results.json`, `before2-results.json`): lookup, Antitone join, short write, three carrier charge deletions, combined public facade projection, and both Bool conversion arms. The facade and Bool compound injections were separated for the final rejection checks. An additional combined Asc/NullLast projection injection and nine individual expression/graph/plan/callable injections survived full pipeline+conformance suites (`execution-sort-projection-before.json`, `final-candidate-results.json`). Four of the nine exercise dormant owner projector branches, not active source execution. Those four are recorded separately from the seven active sort/graph/plan regressions. These owner-local observations are not a source-stable repository qualification receipt.

The attempted residual-order full-workspace run was invalidated by a concurrent source writer and is excluded. The concurrent task's order fix is re-proved here by a separate public-consumer assertion kill. The waypoint-sort candidate is rejected: the existing Through `mid, late` oracle already catches lexical sorting. No redundant waypoint test was added. Join-mode swaps and partition-key clearing were also rejected by existing conformance assertions (`execution-relational-metadata-before.json`); no additional join/partition test was retained. An initial CLI mutation selected zero binary tests; the accepted replay uses the actual library owner. A source-only false predicate fixture missed the raw Bool HIR arm; the separate owner fixture closes that omission.

The first final gate stopped in `static/fmt` before any test run: the committed iterator fix and raw Bool test still needed rustfmt. The failure and unchanged-source snapshot are preserved in `ci-attempt-01.json`/`.log`; formatting was corrected before re-freezing the final source.

The second gate passed nextest (1,307 selected/executed/passed), all five doctests and ten front-door parity cases, then failed architecture policy: the existing P3 corpus test used the pinned `sha2` dev dependency without its exact external-dev declaration. The declaration now pins only surface test use to `=0.10.9` and an empty authored feature list. That unsuccessful gate is preserved as `ci-attempt-02.json`/`.log`; its passing groups are not substituted for the final complete gate.

The final repository-wide authority is `target/sep26-parallel-complete/ci-result.json` with its corresponding `ci.log`: accept only exit zero, equal before/after source snapshots, unchanged HEAD, and complete selected/executed/pass accounting. All source/doc edits precede that run. It is a dirty-source local gate, not clean committed-HEAD qualification, downstream execution, deployment, or physical performance evidence.

## Review inventory and limits

The discovery inventory contains fourteen workspace crates, eighty-one Rust integration test files, and thirty-six source files with unit tests. File names, annotations and current hashes are recorded in `rust-test-file-inventory.json`; runtime discovery and terminal accounting determine actual executed counts. Helpers and fixtures are not counted as independent executable targets.

Stored lane reports in the same target directory distinguish full test-body reads, targeted production-consumer reads, prior unchanged-file review, fixture materialization checks and omitted source-line rereads. Primary inventory files: `syntax-hir-binder-review.md`, `types-modules-sema-inventory.json`, `frontdoors-tools-inventory.json`, `logic-review.json`, `planning-review.md`, `final-pipeline-review.json`, `pipeline-final-two-files-review.json`, and `root-inventory.json`. No crate test target or unit-test source was omitted from inventory. The review is a complete target/contract sweep; it does not claim that every production branch was exhaustively mutated, every fuzz input was explored, or downstream runtime behavior was verified.
