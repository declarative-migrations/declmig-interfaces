# Declarative Migrations — interfaces

Canonical `interfaces` repository for [`declarative-migrations`](https://github.com/declarative-migrations).

- Internal runtimes: Rust, TypeScript, Dart.
- Contract authorities: TypeSpec and JSON Schema/OpenAPI are independent peers.
- TypeSpec emits SQL, Protobuf/gRPC, and wire-client types.
- JSON Schema/OpenAPI emit SQL, client/interface types, and write-client contracts.
- Compare both SQL candidates through DPM catalogs and both type surfaces through
  `declmig.generated-types/v1` manifests.
- Compare SeaORM and Diesel through `declmig.orm-projection/v1` manifests.
- Any discrepancy, missing peer artifact, or tool error must pause; never select
  an automatic winner or silently widen a type.
- Auth: github.com/shared-auth.
- Sync: github.com/opto-sync.
- Telemetry: github.com/ores-otel.
- Flags: github.com/flags-2-env.
- Packages: github.com/zed-pkg.
- Never use React/JSX or webviews.
- Resolve git conflicts semantically; never rebase, stash, or reset.

No function bodies except parse, validate, canonicalize, or fail-closed parity
evaluation.
