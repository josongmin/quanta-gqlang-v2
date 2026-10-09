# 08 Documentation and evidence audit

- Frozen source: QGLang `25fad76cf5b4510244157c2e009b4dc37aa958ff`, tree `99501ba1aabfa37361fccfb7dee388cd91055ee1`. The comparison window `9060e46..25fad76` contains 202 docs-only commits and 117 touched `docs/` paths. Only 31 of those paths remain tracked at HEAD; deleted/returned-to-baseline (`T`) paths are historical evidence, not current documents.
- Read the committed blobs with `git show HEAD:<path>` and the HEAD tree with `git ls-tree`; existing local WIP edits/deletions were inspected separately and not treated as committed repair. No other docs or source were edited by this lane.

## Confirmed committed-HEAD documentation defects

### DOC-01 — resolved QG-DIAG still routed as open, with a dead anchor

- **Current path:** `docs/adr/0002-test-authority-and-adversarial-oracles.md:169-170` says an *unclosed* diagnostics-wire decision remains at `../work/remaining.md#qg-diag`. `docs/adr/0004-sealed-owner-facts-and-bounded-projection.md:115-118` points to the same anchor.
- **Counterevidence:** `docs/adr/0008-private-diagnostics-primary-location-contract.md:3,31` is Accepted and explicitly resolves QG-DIAG as the existing seven-field private V2 wire contract. `docs/work/remaining.md:596-598` says QG-DIAG is resolved; it has no `qg-diag` heading or explicit anchor. Thus both ADR links land at no named section, and ADR-0002 routes a resolved decision into unfinished work.
- **Impact:** A reader using the durable ADR index may reopen a closed contract decision or miss its actual authority. This is a documentation routing/status defect; it does not establish a runtime wire defect.
- **Remedy/status:** Point both ADRs to ADR-0008 and state that wire behavior did not change. The current dirty edits to both ADRs already do this; their integration/commit is pending and was not performed here.

### DOC-02 — top-level active-ticket denominator is wrong

- **Current path:** `docs/README.md:5` describes the substrate ticket index as **19 remaining closeout IDs**.
- **Independent expected count:** `docs/plans/oct-6-substrate-dedup/tickets.md:9-12` explicitly enumerates **18 active closeout IDs** plus resolved QG-DIAG and accepted D4/D5. `docs/work/remaining.md:6,587-588` likewise owns 18 active states. The additional resolved decision must not enter the active denominator.
- **Impact:** The documentation entrypoint inflates remaining scope and conflicts with both owner indexes. No QGLang source defect follows from the count.
- **Remedy/status:** Use 18 active IDs and keep resolved decisions separate. The current dirty `docs/README.md` already does this; its integration/commit is pending and was not performed here.

## Checked and deferred boundaries

- A HEAD-tree link scan of the 31 still-tracked touched docs (34 tracked `docs/*.md` files overall) found no missing **in-repository file** target among ordinary Markdown links. The two `#qg-diag` anchors above were confirmed manually; an anchor scan needs GitHub slug and explicit-anchor handling, so its other raw hits were not accepted as defects. Relative links from these docs to sibling Semantica paths were present on the current filesystem but were not checked against a frozen Semantica commit.
- `docs/adr/0004-sealed-owner-facts-and-bounded-projection.md:74` describes the older Oct-4/Oct-5 maintenance as introducing no wholesale DTO move, while the later HEAD contains `crates/quanta-gqlang-v2-execution-contract/src/lib.rs` and `docs/contracts/frontend-execution-v3.md:7-16` owns the extraction. The ADR declares its older scope at lines 3-5, so this is **ambiguous temporal wording**, not a confirmed current-source contradiction. The dirty ADR edit clarifies chronology; recheck after integration.
- The shared dirty consolidation deletes the Oct-10 closeout packet and most Oct-6 plan files, retains selected short aliases, and rewrites ADR/remaining-work routing. It is not part of HEAD. After that writer commits, repeat link/anchor and retained-ID checks against the resulting SHA; do not treat pre-deletion links or old status tables as current defects.
- `just ci`, `just qualify-local`, hosted CI and downstream product execution were **NOT_RUN** in this documentation lane. The existing HEAD receipt described by the weekly inventory is source-bound local evidence only; this audit did not create a new qualification receipt.
