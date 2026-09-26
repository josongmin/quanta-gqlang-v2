# Sep 26 final test review

Subsequent challenge: [adversarial review](adversarial-review-2026-09-26.md)
found and closed a source-registry diagnostic Display gap that had been held in
P7. Its additional mutation results and final source receipt are separate from
the nine probes below.

Base: `78f477056cfbc726441d298fe243d3d34efb566f` plus the uncommitted second
recheck and final review changes. This patch changes tests, test support and
coverage records. Production semantics and public APIs are unchanged.

## Final changes

| Boundary | Oracle |
|---|---|
| EXPORT semantic budget | Five-step exact success, four-step failure at profile use 88..94, identical no-profile four-step control |
| Composition body retention | Literal Project→Scan node/effect fold; all graph fields against its sealed contract; CHECK and EXPLAIN payloads against their sealed semantic body as well as their checked planning input |
| Request errors | Ten literal schema messages with exact line/column; valid string and numeric IDs echoed exactly |
| Wire schema | Exact success/error envelope, protocol-error and diagnostic key sets; framing/recovery properties remain separate |
| Host callable identity | Independent complete Rule byte vector pins namespace tag 2, lengths and payload, separate from Predicate tag 1 |
| CLI coverage | Exact whole command field rejects malformed brackets/trailing garbage; advertised/covered token-set equality protects binary dispatch coverage |
| Conformance | Explicit NoPlan item variant and a four-case guard for the selected shipped acceptance corpus |
| Process helper | One dedicated CLI test target exercises timeout kill/reap, both large output streams, exact large stdin and unread-input watchdog; binary tests continue to use the same helper |
| Redundant assertions | Removed the explain self-comparison while retaining its independent asymmetric literal family vector |

The helper controls were previously included in five binary test targets.
Running them once removes duplicate subprocess work without changing their
assertions or the shipped-binary checks. Unix-only custody controls do not
prove descendant process-group cleanup. Large stdin is checked byte-for-byte;
failure output reports length/content mismatch without dumping huge vectors.

## Mutation evidence

`target/sep26-final-review/mutation-results.json` records command, original and
mutant source hashes, log hashes, compilation, assertion failure and restoration.
The nine probes are relational node drift, graph depth drift, CHECK pattern
deletion, EXPLAIN evidence drift, schema-message erasure, an extra success-response
key, Rule tag drift, truncated stdin after test relocation, and garbage appended
to the CLI command field. Compiler failures do not count as a kill. The earlier
RA-01~05 and second-recheck mutation results remain separate historical evidence.

## Completion boundary

Final repository-wide validation runs `just ci` after all mutations are restored
and source editing stops. A closeout receipt is valid only when it records exit
0 and identical before/after source snapshots; its terminal
nextest inventory/events determine executed/pass counts. No fixed test count
in an audit document substitutes for that evidence.

The selected positive owner tests run during implementation; their success is
not repository-wide qualification. The final local gate applies to the dirty
source snapshot, not a clean committed HEAD or hosted CI. Runtime/provider/backend
execution and deployment remain downstream-owned.

## Historical held-note disposition

- P6 and P8's concrete optional payload/message/CLI oracle notes are closed by
  this patch. Differential tests remain useful properties backed by separate
  literal oracles; they are not standalone semantic goldens.
- P5's old permutation note is already closed (120 rule orders, 48 diamond
  orders); selector retention is covered in the pipeline owner rails. Lexical
  tie-break is not emitted by production and is not falsely qualified here.
- P4's neutral type-input registry identity and unused capture ordinal are
  contract questions, not a demonstrated authority bypass. The diagnostic
  vocabulary already pins code/Display; no accessor is added just to recompare
  a test-owned constructor value.
- P9's architecture valid fixture is a policy projection; the real workspace
  architecture gate supplies external dependencies/internal features. The
  external-consumer gate includes a compiled public-API inventory beyond its
  source-shape checks. Owned-cache custody and whole-source CI immutability
  are distinct guarantees; this patch does not overstate either.

No retained actionable test-oracle finding in this audit packet remains open.
That does not assert exhaustive mutation coverage of every frontend behavior.
