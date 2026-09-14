# `declmig-runner.rs` contract adoption

The privileged runner consumes a narrow cross-runtime execution contract. The interface repository remains the authority boundary; the runner repository must not invent a second wire model.

## Peer authorities

Maintain an independently authored TypeSpec definition and an independently authored JSON Schema Draft 2020-12 definition for the runner admission envelope. TypeSpec-derived JSON Schema is comparison evidence only. Neither authored authority may be regenerated from the other merely to force a match.

Shared fields use snake_case. Generated language projections are read-only artifacts and must retain the normalized declaration names and required/optional semantics of both peer authorities.

## Required contract surface

The admission model should cover immutable identifiers, canonical plan digest, idempotency identity, requested/scheduled timing, maintenance bounds, execution limits, and fencing token. Secret references, if later required, must be opaque identifiers to an approved secret store and must not permit raw secret material.

The contract must structurally exclude raw DSNs, passwords, bearer tokens, private keys, arbitrary commands, callback URLs, client-selected authorization roles, and unbounded SQL/script payloads.

## TJSV and projection gates

Before publication or runner promotion:

1. validate authored JSON Schema A against Draft 2020-12;
2. compile TypeSpec and produce comparison-only JSON Schema B;
3. compare A and B semantically with TJSV without rewriting either authority;
4. verify stable JCS + SHA-256 fixtures for accepted and rejected envelopes;
5. exercise Protobuf/WIT additions only when they preserve the same semantic contract;
6. verify every generated projection is reproduced from the reviewed authorities and remains read-only.

## Negative fixtures

Include fixtures for unknown fields, credential-smuggling field names, invalid identifiers, uppercase/noncanonical digests, zero/stale fencing tokens, timeout/retry/concurrency overflow, malformed maintenance windows, wrong JSON types, duplicate delivery identities, and unsupported contract versions.

## Compatibility

Changes that add optional bounded fields may be additive. Renames, required-field changes, altered enum semantics, or changed digest/fencing meaning require an explicit contract version and migration plan. The runner must fail closed on contract versions it does not understand.