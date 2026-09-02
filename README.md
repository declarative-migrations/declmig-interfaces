# declmig-interfaces

Canonical data-only contracts for Declarative Migrations.

## Authority

- TypeSpec under `typespec/` is the editable wire-contract source.
- Draft 2020-12 JSON Schema under `schema/` is the immutable validation artifact.
- Rust, TypeScript, and Dart declarations are projections of the same contract.
- Desired database SQL and DPM catalogs remain the persistence authority; this repository does not generate production DDL.
- SeaORM and Diesel are runtime projections, never migration authorities.

## Migration plan v2

`schema/v2/migrationplan.json` replaces the v1 `payload: object` escape hatch with a closed, phase-aware contract. Plans identify the database engine and exact engine version, bind source/desired/rendered SQL digests, and classify every phase by safety, retry, rollback, approval, timeout, lock expectation, and affected resources.

The phase union distinguishes transactional DDL, non-transactional DDL, CockroachDB schema jobs, data backfills, validation, traffic transitions, and destructive cleanup. Language validators add cross-field checks that JSON Schema cannot express safely, including unique phase IDs, contiguous statement ordinals, destructive-phase isolation, and SHA-256 normalization.

The v1 schema remains checked in only as a historical compatibility artifact. New producers and consumers must use protocol version 2 and schema revision `declmig-0002`.

## Generate

The TypeSpec compiler and JSON Schema emitter are pinned exactly in `typespec/package.json`. A resolved lockfile and regenerate-and-diff CI gate are required before a v2 release tag is cut.

```sh
cd typespec
npm install --ignore-scripts --no-audit --no-fund
npm run generate
```

CI should generate into a temporary directory and compare the result with `schema/v2`; generated drift is a release blocker.
