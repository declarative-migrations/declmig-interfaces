# TypeSpec authority

This directory is reserved for the independent top-level TypeSpec authority.
It is intentionally not populated with a partial or unpinned compiler setup.

The first implementation must land with:

- exact TypeSpec compiler and Protobuf emitter versions plus a committed lockfile;
- explicit Protobuf field numbers and reservations;
- a TypeSpec-to-SQL emitter independent of the JSON Schema/OpenAPI SQL mapper;
- proto3/gRPC output;
- Rust, TypeScript, Dart, and other required wire-client output;
- `declmig.generated-types/v1` output;
- deterministic fixtures and negative tests;
- a `pause` certificate until TypeSpec SQL/types agree with the independent
  JSON Schema/OpenAPI SQL/types path.

Do not copy the JSON Schema mapper into the TypeSpec emitter. Shared mapping
code would defeat the cross-check by allowing correlated defects.
