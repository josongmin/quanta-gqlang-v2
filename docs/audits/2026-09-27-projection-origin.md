# D-02: projection origin alignment

Base: `8b2f4df3dadef0435b33cbccd64be064e6ac8ddf`.

The Project subclaim of D-02 is reproduced at both private constructor
boundaries. Starting with a real sealed two-expression projection, the Sema
constructor accepted fewer expression keys than output columns, and the Algebra
constructor accepted fewer typed expressions than output columns. Each baseline
regression selected and failed one test. This is a constructor-contract defect;
the normal source producer builds both vectors together, and no source-language
trigger has been demonstrated.

`RelationOutputContractV2::has_exact_projection_origins` compares the full ordered
sequence of source-qualified expression identities. Iterator equality checks
both length and every element, with no truncating zip or name inference.
Source-field and generated origins cannot compare equal to an expression origin.
Both Sema Project sealing and Algebra Project construction enforce this contract;
existing aggregate projection checks reuse the same predicate. The comparison is
linear, allocation-free, and does not inspect source text or infer types.

Two owner tests retain the real valid projection as a positive control. Both
reject empty, shorter, longer and reversed expressions. Sema additionally rejects
a changed source identity and source-field/generated output origins. The existing
seven Algebra semantic-mutant tests remain passing. Assertions check exact
constructor errors rather than a later unrelated failure.

The helper is an internal-stage API and is not exported by the frontend facade.
Valid syntax, typed output, execution DTOs and digest schema remain unchanged.
The full clean-HEAD qualification receipt is recorded separately.

This closes only D-02's Project origin/count subclaim. Its Join cardinality,
row-type, Aggregate row-type and Semi/Anti correlation statements require their
own producer/consumer analysis and are not established by these tests.
