# Nested digest oracle follow-up

Base: `2d0401924de665227fb9bf45b6bea3996b4ba5f2`.
Owner: pipeline execution-contract encoding. Production behavior and schemas
are unchanged; this closes a proof gap, not a reproduced production defect.

## Acceptance and evidence

- Complete expected source/contract/qualification streams are authored from
  `docs/contracts/frontend-execution-v3.md`, independently of production DTO
  readers, tag functions, writer functions, and the shared TSV.
- Relation fixtures cover every current expression-shape and value-type arm,
  all scalar tags and call-target arms, non-ASCII strings, nested schemas,
  source identity, predicate/expression keys, correlations, partition bindings,
  order keys, and distinct child-node order.
- Composition fixtures cover every producer/body arm, all SAMPLE modes and
  absent/host/local scope, present/absent schema and relation, graph references
  and complete graph witness policy, and ordered multi-item framing.
- Synthetic fixture fields intentionally exercise encoder states directly.
  They are not claims that every field combination is source-admissible.
- Focused command: `cargo test -p quanta-gqlang-v2-pipeline --lib digest_oracle`.
  Baseline and restored source: 2 selected, 2 passed, 0 failed.

## Adversarial controls

Each mutation was applied alone to the production encoder. The same two oracle
tests compiled and ran; each mutant exited 101 with an equality assertion
failure. Compile failures and zero-selection were not counted as kills.

| Mutation | Result |
| --- | --- |
| Omit each relation expression shape | Assertion kill |
| Omit source name | Assertion kill |
| Reverse child-node order | Assertion kill |
| Duplicate composition-binding relation presence byte | Assertion kill |
| Omit SAMPLE scope | Assertion kill |
| Use source digest twice in qualification | Assertion kill |
| Give Union the Record tag | Assertion kill |
| Reverse graph summary-facet order | Assertion kill |

Local raw logs and the replay script are under
`target/digest-oracle-audit/` in the qualified worktree. The restored encoder file
SHA-256 was `9865b6ddc4bf4283dcb64afb91d502d18c219f568db27272a51d6d7f8f009fa7`.
No TSV values were regenerated. Whole-repository qualification is a separate
clean-commit receipt; the focused results above do not substitute for it.

## Boundary

This addresses E2-01/E3-8's absence of independent nested whole-stream expected
bytes and E2-14's provenance ambiguity. It is not an exhaustive Cartesian test
of all enum combinations, a proof of cryptographic collision impossibility, or
runtime qualification. Existing per-field mutation tests, exact source vectors,
tag tests, and facade consumer checks remain complementary evidence.
