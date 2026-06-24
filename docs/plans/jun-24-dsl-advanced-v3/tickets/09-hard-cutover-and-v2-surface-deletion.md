# 09 Hard Cutover And v2 Surface Deletion

## Goal

Make `v3` the default and remove the old architecture as a first-class path.

## Why

If the old direct root-carrier pipeline remains co-equal, the rewrite never
actually closes and future features will continue to fork.

## Deliverables

1. `v3` default frontend path
2. deletion or strict quarantine of legacy direct syntax->typed->IR path
3. removal of obsolete proof rails that only defend the old architecture
4. updated docs and examples that reflect the new default language
5. semantica serving cutover plan pinned against the actual ingress seam

## Acceptable Outcomes

1. delete `v2` direct core entirely
2. keep a compat crate or compat parser only if it is:
   - non-default
   - clearly deprecated
   - unable to block new core design

## Closure Checklist

1. new syntax AST -> HIR -> algebra -> planning path is default
2. advanced expression/query features are represented without legacy carriers
3. old root-local direct core path is deleted or quarantined
4. CLI/LSP/formatter/proof rails are green on `v3`
5. public docs describe `v3` truth, not mixed `v2/v3` ambiguity
6. semantica-side serving uses the sibling contract seam, not frontend-owned
   runtime DTO leakage
7. non-servable `v3` shapes are rejected explicitly rather than downcast into
   legacy `RootSpecV1` forms

## Final Result

This packet is complete only when the repo is structurally a `v3` language
frontend, not a `v2` shell with advanced features bolted on.

## Dependencies

1. requires `00`
2. depends on `01` through `08`
3. must finalize with `10` for honest serving-status claims

## Likely Touch Points

1. legacy direct path owners across `syntax`, `sema`, and `ir`
2. `docs/language/README.md`
3. public examples under `examples/`
4. CLI/LSP/tests that still assume legacy root-local summaries

## Mandatory Worker Outputs

1. explicit inventory of deleted versus quarantined `v2` direct-path surfaces
2. docs and examples updated to describe `v3` as the default
3. final statement of any compat surface that remains and why it cannot block
   `v3`

## Proof Rails

1. run the owning crate rails touched by the cutover
2. extend `crates/quanta-gqlang-v2-conformance/tests/docs_conformance.rs` if
   docs/examples changed
3. extend CLI/LSP frontdoor rails if default behavior changed

## Reject Or Escalate

1. leaving the old direct path co-equal with `v3`
2. silently preserving compat behavior without marking it deprecated/non-default
3. claiming semantica advanced-serving readiness from local repo proof alone
