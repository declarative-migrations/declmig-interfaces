# AttoLang v1 contract projection (candidate only)

`projections/atto/declmig_interfaces.atto` provides typed AttoLang
`HealthContract` and `MigrationPlanHeader` definitions and pure input
guards for the public protocol v1 boundary. The required migration-plan
`payload` is an opaque JSON object and is deliberately **not** mapped to a
lossy AttoLang string. It must be decoded through an admitted contract-aware
implementation before use.

This is a manually reviewed **candidate projection**, not an independent
authority or a generated parity-certified artifact. TypeSpec and JSON
Schema/OpenAPI are still independent peers. The `generated/final` output,
parity receipt, SQL catalogs, and six-peer admission decisions are unchanged.
No absent generator, unchecked type, or failing peer may be treated as passing.

`tests/atto_projection_contract.rs` checks the projection's field names and
required-string invariants against the canonical v1 JSON Schema. The pinned
AttoLang compiler CI compiles the candidate with a smoke main exercising
edge-case guards. Compilation does not prove guest Wasm execution; add
real runtime tests on the AttoLang targets before claiming full Atto support.
