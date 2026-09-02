#![forbid(unsafe_code)]

//! Data-only discrepancy contract shared by generators, DPM, CI, and clients.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Independent top-level contract source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContractAuthority {
    TypeSpec,
    JsonSchemaOpenApi,
}

/// Artifact classes emitted by one of the peer contract lanes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorityArtifactKind {
    Sql,
    TypeManifest,
    Protobuf,
    Grpc,
    WireClient,
    InterfaceTypes,
    Validator,
    WriteClient,
}

/// Why certification could not establish equivalence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorityDiscrepancyKind {
    MissingArtifact,
    InvalidArtifact,
    GeneratorFailure,
    UnsupportedConstruct,
    SemanticMismatch,
}

/// Release-gate decision. There is deliberately no "prefer source" variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorityParityDecision {
    Proceed,
    Pause,
}

/// Immutable identity of a generated artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityArtifactIdentity {
    pub authority: ContractAuthority,
    pub kind: AuthorityArtifactKind,
    pub source_sha256: String,
    pub generator: String,
    pub generator_version: String,
    pub artifact_sha256: String,
}

/// One machine-readable semantic discrepancy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityDiscrepancy {
    pub kind: AuthorityDiscrepancyKind,
    pub artifact_kind: AuthorityArtifactKind,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_spec: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_schema_open_api: Option<Value>,
    pub message: String,
}

/// Shared evidence contract for the peer-authority release gate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityParityReport {
    pub format: String,
    pub contract_revision: String,
    pub decision: AuthorityParityDecision,
    pub artifacts: Vec<AuthorityArtifactIdentity>,
    pub discrepancies: Vec<AuthorityDiscrepancy>,
}

impl AuthorityParityReport {
    pub const FORMAT: &'static str = "authority-parity-report/v1";

    /// Build a fail-closed report from already validated artifact identities
    /// and semantic discrepancies.
    #[must_use]
    pub fn evaluate(
        contract_revision: String,
        artifacts: Vec<AuthorityArtifactIdentity>,
        discrepancies: Vec<AuthorityDiscrepancy>,
    ) -> Self {
        let decision = if discrepancies.is_empty() {
            AuthorityParityDecision::Proceed
        } else {
            AuthorityParityDecision::Pause
        };
        Self {
            format: Self::FORMAT.to_owned(),
            contract_revision,
            decision,
            artifacts,
            discrepancies,
        }
    }

    /// Returns true only when every required peer artifact was supplied by the
    /// caller and semantic comparison found no discrepancy.
    #[must_use]
    pub fn may_publish(&self) -> bool {
        self.decision == AuthorityParityDecision::Proceed && self.discrepancies.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discrepancy_always_pauses() {
        let report = AuthorityParityReport::evaluate(
            "contracts-42".to_owned(),
            Vec::new(),
            vec![AuthorityDiscrepancy {
                kind: AuthorityDiscrepancyKind::SemanticMismatch,
                artifact_kind: AuthorityArtifactKind::Sql,
                path: "$.tables.accounts.columns.email.nullable".to_owned(),
                type_spec: Some(Value::Bool(false)),
                json_schema_open_api: Some(Value::Bool(true)),
                message: "nullability differs".to_owned(),
            }],
        );

        assert_eq!(report.decision, AuthorityParityDecision::Pause);
        assert!(!report.may_publish());
    }

    #[test]
    fn empty_discrepancy_set_proceeds() {
        let report = AuthorityParityReport::evaluate(
            "contracts-42".to_owned(),
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(report.decision, AuthorityParityDecision::Proceed);
        assert!(report.may_publish());
    }
}
