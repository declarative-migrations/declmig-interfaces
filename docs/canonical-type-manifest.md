# Canonical type and operation manifest

`canonical-type-manifest/v1` is the neutral comparison target for the TypeSpec and JSON Schema/OpenAPI lanes. It is not an authoring source and must never be edited to resolve a disagreement.

Each adapter emits the manifest directly from its own top-level source and records exact source, generator, configuration, and artifact identities. The comparison excludes `authority`, generator metadata, timestamps, and source-specific comments. It sorts type, field, variant, operation, and error collections only where ordering has no declared semantic meaning.

The following differences always produce `pause`:

- missing or extra types, fields, variants, operations, or errors;
- scalar/base type and integer-width differences;
- required versus optional and nullable versus non-nullable differences;
- decimal precision/scale and numeric-bound differences;
- identifier, timestamp, timezone/offset, binary, and format differences;
- default values and validation constraints;
- enum serialized values and union discriminators;
- read-only, write-only, and client read/write direction;
- request, response, error, transport, path/method, and streaming shape.

Language-level names may differ only through an explicit serialized-name mapping that is present in both manifests. An equivalence rule must be narrowly scoped, versioned, and reviewed; it cannot remove a semantic field from comparison.

The DPM parity gate compares this manifest separately from SQL catalogs. Equal wire types do not prove equal database semantics, and equal SQL catalogs do not prove equal client contracts. Both checks must pass before Diesel/SeaORM generation begins.
