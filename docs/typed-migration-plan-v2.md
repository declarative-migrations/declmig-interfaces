# Typed migration-plan v2 salvage boundary

## Status

`STOPPED_FOR_EVALUATION` for publication and fleet adoption.

The Rust module `migration_plan_v2` preserves a strict typed migration-plan
contract recovered from the superseded `codex/typed-migration-plan-v2` branch.
It is intentionally additive: stable protocol v1 remains available and no v1
payload is reinterpreted as v2.

## Contract shape

The v2 envelope records:

- format, plan identity, logical revision, database engine, and exact engine
  version;
- source-catalog, desired-catalog, and rendered-SQL SHA-256 digests;
- ordered typed phases for transactional DDL, non-transactional DDL,
  CockroachDB schema jobs, resumable data backfills, validation, traffic
  transitions, and destructive cleanup;
- explicit safety, retry, rollback, approval, timeout, lock, change,
  precondition, and postcondition metadata.

Unknown fields, missing phases, duplicate phase IDs, non-lowercase SHA-256
values, zero timeouts or batch sizes, destructive changes outside a destructive
cleanup phase, missing destructive approval/backup evidence, malformed checks,
and CockroachDB-only phases in PostgreSQL plans fail closed.

### Statement integrity

Every migration statement carries an exact lowercase SHA-256 digest of its SQL
bytes. Validation now recomputes that digest rather than checking only its
shape. Changing SQL without changing the digest, or replacing the digest with a
different valid-looking 64-character hexadecimal value, returns the stable
fail-closed error `statement.sha256 must match statement.sql`.

This binds statement order and content inside each typed phase. The top-level
`rendered_sql_sha256` remains a separately supplied release-evidence digest
because the canonical rendering may include separators and planner metadata not
recoverable from the individual statement array alone. Its provenance and
recomputation remain mandatory in the DPM planner/release gate.

## Independent authority requirement

This Rust type is not contract authority by itself. Before v2 may become a
published cross-language protocol, the repository must contain and compile two
independent top-level source lanes:

```text
TypeSpec source
  -> TypeSpec semantic IR
  -> migration-plan wire/client manifest
  -> Protobuf/gRPC where applicable
  -> supported wire clients
```

paired with:

```text
JSON Schema Draft 2020-12 + OpenAPI sources
  -> JSON/OpenAPI semantic IR
  -> validators and language interfaces/types
  -> write-client request/response/error surfaces
```

A TypeSpec-emitted JSON Schema/OpenAPI document is diagnostic evidence only and
may not replace, feed, or certify the independently authored JSON
Schema/OpenAPI lane. The inverse rule applies to generated TypeSpec.

The old branch's generated `schema/v2/migrationplan.json` and generated clients
are therefore not copied into the production roots by this salvage. Their
semantics may be used as review input, but both authorities must be authored,
compiled, normalized, and compared independently at current revisions.

## Required promotion evidence

Publication remains blocked until CI proves:

1. TypeSpec and JSON Schema/OpenAPI independently describe every v2 field,
   variant, required/optional/nullable distinction, discriminator, constraint,
   and operation boundary;
2. canonical generated-type manifests from both lanes compare equal;
3. TypeScript, Dart, Rust, Gleam, Go, and other supported generated clients pass
   shared positive and negative fixtures;
4. the TypeSpec Protobuf/gRPC lane preserves stable field numbers,
   reservations, and streaming cardinality where used;
5. the typed DPM planner and executor consume the same semantic envelope and
   recompute the top-level rendered-SQL evidence digest;
6. generated artifacts carry exact source/compiler/output digests; and
7. every discrepancy produces one deterministic report and remains
   `STOPPED_FOR_EVALUATION` rather than selecting an automatic winner.

## Compatibility

Protocol v1 remains exported as `protocol::MigrationPlan` with
`PROTOCOL_VERSION = "1"`. Typed v2 is exported as `MigrationPlanV2` with
`MIGRATION_PLAN_PROTOCOL_VERSION = "2"` and
`MIGRATION_PLAN_FORMAT = "ores.migration-plan/v2"`.

Callers must select the version explicitly. There is no silent fallback,
upgrade, downgrade, or opaque-payload coercion between v1 and v2.
