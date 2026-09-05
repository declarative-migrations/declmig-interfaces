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

## Repository-local Git worktrees

- Create or use a Git worktree only when the human operator explicitly authorizes it for the current task. Concurrency or a dirty checkout is not permission by itself.
- Put every authorized worktree at `<repository-root>/tmp/worktrees/<name>`; from the repository root, use `./tmp/worktrees/<name>`. Never place worktrees beside repositories or organization directories.
- Keep `tmp`, `temp`, `tmp/worktrees`, and `temp/worktrees` ignored in the repository-root `.gitignore`. Do not commit files from those directories.
- Relocate or remove a worktree only when the operator explicitly requests it. Before removal, preserve and publish intended changes, verify its commit is represented on the target branch, and confirm there are no tracked, untracked, ignored-sensitive, or in-use files that must survive. Remove it with `git worktree remove <path>` without `--force`; never delete a worktree directory with `rm`.
