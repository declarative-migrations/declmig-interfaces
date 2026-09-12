# Typed migration-plan v2 current-main salvage evidence

## Provenance

The typed migration-plan v2 model was recovered from stale pull request #1,
whose source head was based 27 commits before the current peer-authority branch.
The stale tree was not merged wholesale. The additive Rust validation surface
was reconstructed from current `main@242a95902c40d6b9947e533680755b50806cc9a3` on
pull request #11.

The formatting reconciler validated the reconstructed model, applied only
canonical Rust formatting and two strict-Clippy repairs, then self-deleted and
committed the reviewed result as
`5773abe4a7e2c3fe3bbcc643f33f3cdd0cbf02f3`.

## Preserved compatibility

- stable protocol v1 remains exported as `protocol::MigrationPlan` with
  `PROTOCOL_VERSION = "1"`;
- typed protocol v2 is additive and explicitly selected through
  `MigrationPlanV2`, `MIGRATION_PLAN_PROTOCOL_VERSION = "2"`, and
  `ores.migration-plan/v2`;
- v1 payloads are rejected by the v2 decoder rather than silently upgraded or
  reinterpreted;
- unknown fields and unsafe or internally inconsistent phase metadata fail
  closed.

## Authority boundary

The stale branch's TypeSpec-emitted JSON Schema and generated clients were not
copied into the independent JSON Schema/OpenAPI production source. TypeSpec and
JSON Schema/OpenAPI remain separately authored top-level authorities.

The Rust model is therefore an implementation and validation surface, not a
cross-language release certificate. Publication remains
`STOPPED_FOR_EVALUATION` until independent source roots, canonical semantic
manifests, generated clients, shared fixtures, and Proto/gRPC compatibility all
converge at exact revisions.

## Current certification requirement

Fresh permanent workflows must pass on the exact human-authored head that
contains this evidence record. The pull request remains draft after those tests
because the missing independent source and client lanes are architectural
promotion blockers, not CI formatting defects.

No production database, migration, package/client publication, provider, DNS,
credential, or deployment mutation is authorized by this record.
