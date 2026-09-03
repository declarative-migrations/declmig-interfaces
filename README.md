# declmig-interfaces

Data-only contracts for Declarative Migrations.

## Peer authorities

TypeSpec and JSON Schema/OpenAPI are independent top-level authorities:

- TypeSpec generates a SQL candidate, Protobuf/gRPC, and wire-client types.
- JSON Schema/OpenAPI generate a SQL candidate, client/interface types, and
  write-client contracts.
- DPM compares the independently generated SQL through normalized database
  catalogs.
- Generated types are compared through the versioned semantic manifest in
  `schema/v1/generated-type-manifest.json`.
- SeaORM and Diesel projection parity is reported in the same fail-closed
  certificate through `schema/v1/orm-projection-manifest.json`.

Any discrepancy, missing peer artifact, or generator error returns
`decision=pause`. No source or ORM wins automatically. See
[`docs/peer-contract-authorities.md`](docs/peer-contract-authorities.md).

JSON Schema contracts remain under `schema/v1`. Generated TypeScript and Dart
surfaces remain under `generated/` and must stay types-only. TypeSpec source and
its pinned generation toolchain will live under `typespec/`; until that peer
path exists and passes certification, release parity is deliberately paused.
