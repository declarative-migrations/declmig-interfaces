# Peer contract authorities

## Policy

TypeSpec and JSON Schema/OpenAPI are separate, top-level authorities. Neither is
a generated compatibility view of the other.

```text
TypeSpec
  ├── SQL candidate
  ├── Protobuf / gRPC
  └── wire-client types

JSON Schema + OpenAPI
  ├── SQL candidate
  ├── client/interface types
  └── write-client contracts
```

Every release generates both paths independently, then compares their semantic
outputs:

- SQL is materialized and compared through DPM's normalized database catalogs.
- Client/interface types are reduced to `declmig.generated-types/v1` manifests.
- A release cannot continue unless the SQL catalogs and generated-type semantic
  models agree.

A discrepancy, missing peer generator, invalid artifact, or generator failure
produces a `pause` decision. Reviewers inspect the evidence and correct a source,
a mapping, or an explicitly scoped exception. Tooling never selects TypeSpec,
JSON Schema, or OpenAPI as an automatic winner.

## Independence requirement

Cross-check value depends on implementation diversity. The two paths may share:

- the manifest envelope and validation schemas;
- fixture values and expected business invariants;
- DPM as the neutral SQL-catalog comparison layer;
- release-evidence hashing and storage.

They must not share the source-to-SQL or source-to-type mapping implementation.
Otherwise a single mapper defect could appear as agreement.

## Canonical generated-type manifest

`schema/v1/generated-type-manifest.json` defines the comparison IR. Generators
must produce deterministic ordering for model, field, enum, union, service, and
operation arrays. Provenance fields are retained but excluded from semantic
equality by the certifier; the `semanticModel` value is compared exactly after
object-key canonicalization.

The IR distinguishes:

- required from optional;
- nullable from non-nullable;
- scalar width and wire encoding;
- lists and maps from scalar references;
- enums from discriminated unions;
- request/read/write visibility;
- HTTP from gRPC operations and streaming modes.

A generator that cannot represent a declared concept must fail. It must not
silently widen the type to `unknown`, `object`, `serde_json::Value`, or an
unconstrained map.

## SQL mappings

Each authority produces a complete candidate SQL schema for each certified
engine. SQL annotations/extensions belong to the authority that emitted them.
The comparison is not a textual diff: DPM materializes both candidates against
the same exact PostgreSQL or CockroachDB version and requires an empty catalog
diff.

Concepts that cannot be represented in both sources—such as a database-specific
index method or policy—must be modeled as explicit extensions on both sides or
recorded as a narrow, reviewed exception. They cannot be ignored globally.

## Protobuf and gRPC

The TypeSpec path emits proto3 and gRPC service definitions. Protobuf field
numbers are permanent compatibility identifiers and must be explicitly assigned
and reserved after removal. The generated Protobuf/gRPC surface is also reduced
to the generated-type manifest so it can be checked against the JSON
Schema/OpenAPI client/interface surface at the semantic level.

## Current discrepancy

The repository currently contains JSON Schema and generated language surfaces,
but no checked-in TypeSpec source, pinned TypeSpec toolchain, independent SQL
emitter, or Protobuf/gRPC output. `MigrationPlan.payload` is also unconstrained.
Under this policy the current state is intentionally **paused**, not implicitly
JSON-Schema-authoritative. The related GitHub and Linear work items track the
missing TypeSpec path and the removal of the untyped payload.
