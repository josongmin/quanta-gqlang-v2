# Non-empty blocked failure authority and residual documentation audit

Base: `2b8a77a7e932d0db9830b612b9f85897ab806c59`.

## Finding, acceptance and change

E3-5 was a loss of type authority, not a reproduced source-input defect. Both
production producers already supplied `FrontendFailureV2`, whose private list
cannot be empty, but the pipeline copied that list into a plain Vec for the
blocked item/rule carriers. Compile reconstructed the non-empty type and used
`pipeline.document.missing_item_result`, source 0, span (0,0) if it failed. That
fallback described neither the real owner failure nor a real assembly failure.

Acceptance: carry the existing non-empty owner failure through both blocked
paths; preserve original diagnostic identity, ordering, compile deduplication,
analysis item projection and pre-fanout diagnostic counting; remove the fabricated
fallback without a replacement error code or panic. The two private carriers now
hold FrontendFailureV2 directly. Compile wraps that value; Sema's consuming
`deduplicated` method preserves non-emptiness while retaining the existing
adjacent-full-equality dedup policy. Analysis materializes display slices only
at its output boundary. No execution encoding or public diagnostic code changes.
The additive owner method does not expose a mutable or empty carrier.

## Verification

- Three pipeline regression tests preserve exact primary/secondary locations,
  stage, severity, code and message through item/rule compile failures and item
  analysis. Rule and item failures have distinct codes to detect lost precedence.
- Two identical owner diagnostics count once for the analysis cap. A zero cap
  rejects. One rule-program failure projects onto both real rule members after
  canonical counting, so a cap of one still admits the blocked analysis result.
- One Sema regression rejects an empty constructor input, preserves a singleton
  and pins adjacent deduplication with secondary-span-distinct equal-key ties.
- Three compiling mutants drop rule failure precedence, omit compile dedup or
  erase analysis diagnostics. All fail runtime assertions; the restored three
  pipeline regressions pass. Compile errors do not count as killed mutants.
- The initial unchanged pipeline suite passed 44 tests; the added focused rails
  passed 3 pipeline tests and 1 Sema test. Full clean-HEAD qualification is a
  separate receipt, not inferred from these owner checks.

Raw mutation logs live under `target/blocked-failure-audit/` in the owner
worktree. Structural non-emptiness follows from private fields and the two checked
constructors; no claim is made that an empty blocked stage was reachable before.

## Current-source residual dispositions

| Historical finding | Current disposition |
| --- | --- |
| A1-1 dependency removal | Intended allowlist maximum, not a required-dependency inventory. Clarified policy; no gate relaxation or invented dependency-retention constraint. |
| A1-2 facade owner internals | Corrected docs: curated owner-defined values and plan results are public; owner modules and proof constructors are not re-exported. Removed stale entry-point count. |
| C-15 provisional partition resolution | Clarified that Local records candidate binding identity; later checked aggregate context grants access. No semantic acceptance change. |
| C-16 signed-zero/ordering | Documented finite-only domain and canonical positive zero. Bit equality/hash and total_cmp agree on that normalized domain. |
| E1-01 reserved RuleProgram | Reserved item-family tag documented in enum and contract; no producer or execution capability claimed. |
| E1-02 plan projection | Rustdoc explicitly distinguishes summary tags from full semantic policy and rule-program payloads; digest coverage unchanged. |
| E3-2 code ownership | Sema owns transaction diagnostics, diagnostic.rs owns bridges/assembly codes, compile.rs owns open-analysis codes; corrected module description. |
| E3-3 stage/prefix | Removed false prefix derivability claim: reporting stage can relay another owner's code. |
| E3-4 canonicalization | Documented stable sort with adjacent full-struct dedup as Pipeline policy. Secondary-span ties preserve input order; arbitrary separated duplicates are not promised removed. |
| E3-5 empty blocked fallback | Removed structurally as described above. |
| E3-6 invalid closed-ID sentinels | Dormant defensive branches remain. IDs have private representations and owner registry tests exhaustively check issued IDs' spellings. No current source-to-sentinel path demonstrated; do not claim those arms executed or removed. |

This report is not a blanket closure of all historical A/C/E suggestions.
Optional allocation/refactor suggestions and downstream runtime claims are not
frontend defects merely because an old report listed them. A separate current
source disposition is required for any remaining acceptance requirement.
