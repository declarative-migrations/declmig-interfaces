# Peer-authority certificate identity and validity

`declmig.peer-authority-certification/v1` is a fail-closed release input, not a
serialized authorization flag.

A consumer must deserialize the certificate through the typed
`declmig-interfaces` model and call `is_continue_for(expected_inputs, now)`.
This recomputes the three required comparison results, evidence identities,
policy, complete input identities, and validity window. It also requires the
certificate inputs to equal the exact release candidate inputs, so a valid
certificate for an older logical revision or different artifact cannot be
reused.

## Bound identities

An all-pass certificate binds:

- logical contract revision and exact database engine/version;
- normalized desired-catalog and DPM digests;
- human-authored TypeSpec and JSON Schema/OpenAPI source digests;
- independently generated `SQL_T` and `SQL_J` digests;
- Protobuf descriptor and gRPC surface digests;
- TypeSpec wire-client and JSON Schema/OpenAPI type/write-client digests;
- SeaORM and Diesel projection digests; and
- exact TypeSpec, JSON Schema/OpenAPI, SeaORM, and Diesel generator digests.

Digest identities are canonical lower-case SHA-256 values. A pause certificate
may retain `null` for missing artifacts so the absence remains machine-readable;
a continue decision requires every identity.

## Time and immutable identity

Certificates carry an inclusive issue time and exclusive expiry time as Unix
seconds. The validity window must be non-empty, and consumers must reject a
certificate before its issue time or at/after expiry.

`canonical_sha256()` sorts comparison evidence by comparison kind before typed
JSON serialization and returns a stable semantic certificate digest. It does not
hash arbitrary input whitespace. Release packagers bind this digest separately
and reject a mismatch.

Changing `decision`, `decisionEligible`, policy, comparison status, evidence,
input identity, or time fields cannot make a certificate pass because consumers
recompute the gate from the typed content.
