# Retired V1 algebra laws

The V1 normalization API and its unrepresentable operators are gone. This
file remains only as the stable target of the source-level disposition note in
the surface closed-stage test.

The current algebra boundary accepts only sealed V2 semantic relations. Its
public lowering entry points are `lower_relation_item` and
`lower_nested_relation_v2`; the supported relation stages are `where`,
`project`, `join`, `cross`, `group`, and `order`.

The former snippets for `normalize_relation`, `NormalizeOptionsV1`,
`CompositionV1`, `union`, `limit`, and `apply` were deliberately removed. They
cannot compile, do not specify current behavior, and must not be used as test
templates. Current behavior is specified by the executable V2 algebra and
surface conformance suites.

Historical migration records remain under `docs/plans/`; they are not a
current API or test contract.
