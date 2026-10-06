# Substrait source audit for the code-query contract

> 고정 SHA의 소스 감사 기록이다. 로컬 구현·계획 언급은 감사 시점 관찰이며, 현행 판단은 [근거 인덱스](../2026-10-04-reference-source-review.md)와 [RFC](../../rfcs/2026-10-04-code-query-dsl-and-substrate.md)를 따른다.

- Scope: plan/type/extension and consumer-support contract; Substrait is not an execution engine.
- Clone: `/Users/songmin/Documents/code-new/.reference-audits/2026-10-04-code-query/substrait`
- Source: `ab7eff6bf7589d4bd94b26eccd3cb040d45efea2`; shallow full HEAD checkout, clean at capture.
- Proof: source inspection. No Substrait build or test execution.

## Source findings and local decisions

| Source mechanism | Exact pinned evidence | Decision for QGLang/Semantica |
| --- | --- | --- |
| Plan version is a first-class envelope field; a small version-only message can diagnose incompatible plans before full deserialization | [plan.proto:32–81](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/proto/substrait/plan.proto#L32-L81) | V4 needs an identifiable contract/digest domain and explicit stale-version rejection. Do not infer compatibility from a successful decode or from matching repository versions. |
| Semantic extensions and advisory optimizations have different treatment | [extensions.proto:80–88](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/proto/substrait/extensions/extensions.proto#L80-L88) | Required flow policy, set/bag/null laws, completeness and typed entity domains must be understood or rejected. Statistics and preferred access paths can be ignored. A hard host budget is a mandatory admission/execution constraint, never an ignorable optimization hint. |
| Relation anchors are plan-wide unique and explicitly bind correlated references | [algebra.proto:16–36](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/proto/substrait/algebra.proto#L16-L36), [OuterReference:1765–1779](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/proto/substrait/algebra.proto#L1765-L1779) | A03 must validate unique logical operator IDs, every edge/reference, row-binding scope and output schema. Source-stage provenance cannot double as operator identity. Correlated references need explicit owners even when a relation is shared. |
| Read predicates have exact and best-effort fields; filter references precede projection | [algebra.proto:116–122](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/proto/substrait/algebra.proto#L116-L122), [logical_relations.md:30–41](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/site/docs/relations/logical_relations.md#L30-L41) | Provider pushdown is not a boolean supported flag. Preserve a residual filter for inexact pushdown, retain columns needed by that filter, and prohibit limit pushdown that discards rows before residual evaluation. Index search windows do not authorize exact predicate removal. |
| Join match condition and post-join output filter are different fields | [algebra.proto:273–303](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/proto/substrait/algebra.proto#L273-L303) | A06/A08 need differential cases separating join matches from output filters, especially semi/anti and future outer/mark joins. Equal-key matching plus a later filter is not automatically equivalent. Unsupported join kinds remain typed refusals. |
| Extension type/function identities are namespace and plan-local anchor mappings; a function is referenced by signature | [extensions.proto:13–76](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/proto/substrait/extensions/extensions.proto#L13-L76) | Pin model/module signatures, nominal domain and semantics in the frontend contract, and validate mappings at admission. A familiar provider/model name or stringified entity is not compatibility proof. QGLang's locked pack digest is a stronger local binding than a name alone. |
| A dialect declares supported relation kinds and enum subsets; tests check the declaration schema against protobuf | [relations_test.yaml:6–32](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/dialects/tests/relations_test.yaml#L6-L32), [tests/README.md:7–27](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/tests/README.md#L7-L27) | A03/A05 need a versioned executable support profile: relation, expression, type, join/flow policy and result status. Gate the profile against producer variants and actual admission/dispatch; schema version alone does not say what can execute. |
| Typed test cases specify exact values/errors and function options; support controls which cases apply | [tests/README.md:90–111](https://github.com/substrait-io/substrait/blob/ab7eff6bf7589d4bd94b26eccd3cb040d45efea2/tests/README.md#L90-L111) | Use a shared declarative semantic corpus with independent expected tuples/errors, then run the frontend, reference evaluator and production consumer against it. Do not count only decoder round trips or enum coverage. |

## Compatibility decision

Substrait pursues broad cross-version compatibility and staged field migration. QGLang/Semantica explicitly permit a pre-release breaking cutover with one reader. These are different deployment constraints. Adopt explicit semantic support and version identity, while keeping the agreed single V4 contract; a Substrait-style dual writer/reader is not required for this project.

Adopting the whole protobuf plan would still require extensions for code entity/source authority, recursive programs, flow models, typed findings and effect grants. This audit supports the contract patterns above, not replacing the QGLang IR with Substrait or adding a runtime proxy.

## Required RFC corrections

1. Make semantic requirements and advisory optimizer hints separate fields/policies.
2. Add an independent logical-program structural validator to A03/A05: unique IDs, valid references, correlation scope, arity/types and schema. Ordinary relation edges form a valid acyclic computation graph; recursion is represented through the explicitly admitted RuleProgram.
3. Add versioned consumer support profile and variant-to-admission/dispatch drift checks.
4. Add exact/inexact/residual pushdown and join-condition-versus-post-filter laws to A02/A06/A08.
5. Keep cross-system source publication in Semantica runtime authority; neither version tags nor extension names provide source custody.
