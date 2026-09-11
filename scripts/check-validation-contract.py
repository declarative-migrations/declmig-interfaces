#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = ROOT / "validation" / "public-contracts.v1.json"
TYPESPEC = ROOT / "validation" / "typespec" / "validation.tsp"
BINDINGS = ROOT / "validation" / "route-bindings.v1.json"
PUBLIC_MODELS = {"RequestMeta", "PageQuery", "ProblemDetails"}
PUBLIC_DECLARATIONS = PUBLIC_MODELS | {"PublicValidationContract"}
PRIVATE_NAMES = {"TrustedActor", "ServerRequestContext", "InternalCommand"}

schema = json.loads(PUBLIC.read_text())
typespec = TYPESPEC.read_text()
bindings = json.loads(BINDINGS.read_text())
assert schema["contractVersion"] == "ores.validation.v1"
assert schema["visibility"] == "public"
assert set(schema["$defs"]) == PUBLIC_DECLARATIONS
assert schema["$ref"] == "#/$defs/PublicValidationContract"
assert bindings["contractVersion"] == schema["contractVersion"]
assert bindings["routeAuthority"] == schema["routeAuthority"]
for name in PUBLIC_MODELS:
    assert f"model {name}" in typespec, f"missing TypeSpec peer model: {name}"
assert "union PublicValidationContract" in typespec, "missing TypeSpec peer union: PublicValidationContract"
for name in PRIVATE_NAMES:
    assert name not in json.dumps(schema), f"private model leaked: {name}"
    assert name not in typespec, f"private model leaked: {name}"
for binding in bindings["bindings"]:
    assert binding["operationId"].strip()
    for field in ("requestSchema", "responseSchema", "errorSchema"):
        if field in binding:
            assert binding[field] in PUBLIC_MODELS
print("public validation authorities and route bindings are coherent")
