# Examples Index

This index lists the current read-only v3 frontend example sources. Inclusion
proves syntax/format and documented open-analysis behavior, not runtime support.

Query roots:

1. `examples/select/basic.qglang`
2. `examples/find/taint.qglang`
3. `examples/check/hybrid_rule.qglang`
4. `examples/explain/why_no_result.qglang`

Interactive roots:

1. `examples/show/capabilities.qglang`
2. `examples/describe/rule_detail.qglang`
3. `examples/sample/retrieval.qglang`
4. `examples/export/sarif.qglang`
5. `examples/compare/semantic_delta.qglang`

Declaration roots:

1. `examples/view/catalog_users.qglang`
2. `examples/rule/shell_exec.qglang`
3. `examples/model/embeddings_code.qglang`
4. `examples/pack/security_default.qglang`
5. `examples/profile/strict_review.qglang`
6. `examples/gate/release_gate.qglang`

Meta roots:

1. `examples/use_pack/secure_pack.qglang`
2. `examples/require_certified/strict_only.qglang`

Composition family (the clause-level `LET`/`WITH` family; a composition of roots,
not one of the 17 root families):

1. `examples/composition/let_with_find.qglang`

Notes:

1. all examples are read-only
2. no runtime truth is implied by example inclusion
3. experimental catalog/detail surfaces remain typed frontend examples only
4. CLI `fmt --file` succeeds for these sources; CLI `query --file` uses open
   authority and reports `incomplete` with exit code 1 rather than executing
