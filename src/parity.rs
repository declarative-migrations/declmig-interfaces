//! Fail-closed evaluation of peer contract and ORM comparison evidence.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const PEER_AUTHORITY_CERTIFICATION_FORMAT: &str =
    "declmig.peer-authority-certification/v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonKind {
    SqlCatalog,
    GeneratedTypes,
    OrmProjection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonStatus {
    Pass,
    Discrepancy,
    Missing,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParityDecision {
    Continue,
    Pause,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComparisonEvidence {
    pub kind: ComparisonKind,
    pub left: String,
    pub right: String,
    pub status: ComparisonStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PeerAuthorityPolicy {
    pub automatic_winner: bool,
    pub on_discrepancy: String,
    pub required_comparisons: Vec<String>,
}

impl Default for PeerAuthorityPolicy {
    fn default() -> Self {
        Self {
            automatic_winner: false,
            on_discrepancy: "pause-and-evaluate".to_owned(),
            required_comparisons: vec![
                "typespec-vs-json-schema-openapi-sql-catalog".to_owned(),
                "typespec-vs-json-schema-openapi-generated-types".to_owned(),
                "seaorm-vs-diesel-orm-projection".to_owned(),
            ],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PeerAuthorityInputs {
    pub dpm_sha256: Option<String>,
    pub typespec_sql_sha256: Option<String>,
    pub json_schema_open_api_sql_sha256: Option<String>,
    pub typespec_types_sha256: Option<String>,
    pub json_schema_open_api_types_sha256: Option<String>,
    pub sea_orm_projection_sha256: Option<String>,
    pub diesel_projection_sha256: Option<String>,
}

impl PeerAuthorityInputs {
    fn all_present_and_valid(&self) -> bool {
        [
            self.dpm_sha256.as_deref(),
            self.typespec_sql_sha256.as_deref(),
            self.json_schema_open_api_sql_sha256.as_deref(),
            self.typespec_types_sha256.as_deref(),
            self.json_schema_open_api_types_sha256.as_deref(),
            self.sea_orm_projection_sha256.as_deref(),
            self.diesel_projection_sha256.as_deref(),
        ]
        .into_iter()
        .all(|value| value.is_some_and(is_sha256))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PeerAuthorityCertification {
    pub format: String,
    pub decision: ParityDecision,
    pub decision_eligible: bool,
    pub policy: PeerAuthorityPolicy,
    pub comparisons: Vec<ComparisonEvidence>,
    pub inputs: PeerAuthorityInputs,
}

impl PeerAuthorityCertification {
    /// Builds a certificate without selecting a winning authority.
    ///
    /// The decision is `Continue` only when there is exactly one successful
    /// result for every required comparison kind and no extra non-pass result.
    /// Duplicate, missing, discrepant, or errored evidence fails closed.
    #[must_use]
    pub fn evaluate(
        comparisons: Vec<ComparisonEvidence>,
        inputs: PeerAuthorityInputs,
    ) -> Self {
        let decision_eligible = comparison_set_passes(&comparisons)
            && comparisons.iter().all(|item| {
                item.evidence_sha256
                    .as_deref()
                    .is_some_and(is_sha256)
            })
            && inputs.all_present_and_valid();

        Self {
            format: PEER_AUTHORITY_CERTIFICATION_FORMAT.to_owned(),
            decision: if decision_eligible {
                ParityDecision::Continue
            } else {
                ParityDecision::Pause
            },
            decision_eligible,
            policy: PeerAuthorityPolicy::default(),
            comparisons,
            inputs,
        }
    }

    /// Recomputes release eligibility instead of trusting serialized flags.
    #[must_use]
    pub fn is_continue(&self) -> bool {
        self.format == PEER_AUTHORITY_CERTIFICATION_FORMAT
            && self.decision == ParityDecision::Continue
            && self.decision_eligible
            && self.policy == PeerAuthorityPolicy::default()
            && comparison_set_passes(&self.comparisons)
            && self.comparisons.iter().all(|item| {
                item.evidence_sha256
                    .as_deref()
                    .is_some_and(is_sha256)
            })
            && self.inputs.all_present_and_valid()
    }
}

fn comparison_set_passes(comparisons: &[ComparisonEvidence]) -> bool {
    let required = BTreeSet::from([
        ComparisonKind::SqlCatalog,
        ComparisonKind::GeneratedTypes,
        ComparisonKind::OrmProjection,
    ]);
    let observed: BTreeSet<_> = comparisons.iter().map(|item| item.kind).collect();
    let unique = observed.len() == comparisons.len();
    let complete = observed == required;
    let all_pass = comparisons
        .iter()
        .all(|item| item.status == ComparisonStatus::Pass);
    let sql_exit_is_zero = comparisons.iter().all(|item| {
        item.kind != ComparisonKind::SqlCatalog || item.tool_exit_code == Some(0)
    });

    unique && complete && all_pass && sql_exit_is_zero
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{
        ComparisonEvidence, ComparisonKind, ComparisonStatus, ParityDecision,
        PeerAuthorityCertification, PeerAuthorityInputs,
    };

    fn evidence(kind: ComparisonKind, status: ComparisonStatus) -> ComparisonEvidence {
        ComparisonEvidence {
            kind,
            left: "left".to_owned(),
            right: "right".to_owned(),
            status,
            tool_exit_code: (kind == ComparisonKind::SqlCatalog).then_some(0),
            message: None,
            evidence_sha256: Some("a".repeat(64)),
        }
    }

    fn inputs() -> PeerAuthorityInputs {
        let digest = Some("b".repeat(64));
        PeerAuthorityInputs {
            dpm_sha256: digest.clone(),
            typespec_sql_sha256: digest.clone(),
            json_schema_open_api_sql_sha256: digest.clone(),
            typespec_types_sha256: digest.clone(),
            json_schema_open_api_types_sha256: digest.clone(),
            sea_orm_projection_sha256: digest.clone(),
            diesel_projection_sha256: digest,
        }
    }

    #[test]
    fn all_three_independent_comparisons_are_required() {
        let certification = PeerAuthorityCertification::evaluate(
            vec![
                evidence(ComparisonKind::SqlCatalog, ComparisonStatus::Pass),
                evidence(ComparisonKind::GeneratedTypes, ComparisonStatus::Pass),
                evidence(ComparisonKind::OrmProjection, ComparisonStatus::Pass),
            ],
            inputs(),
        );

        assert_eq!(certification.decision, ParityDecision::Continue);
        assert!(certification.is_continue());
    }

    #[test]
    fn discrepancy_pauses_without_choosing_a_winner() {
        let certification = PeerAuthorityCertification::evaluate(
            vec![
                evidence(ComparisonKind::SqlCatalog, ComparisonStatus::Pass),
                evidence(
                    ComparisonKind::GeneratedTypes,
                    ComparisonStatus::Discrepancy,
                ),
                evidence(ComparisonKind::OrmProjection, ComparisonStatus::Pass),
            ],
            inputs(),
        );

        assert_eq!(certification.decision, ParityDecision::Pause);
        assert!(!certification.decision_eligible);
        assert!(!certification.policy.automatic_winner);
    }

    #[test]
    fn missing_or_duplicate_evidence_pauses() {
        let missing = PeerAuthorityCertification::evaluate(
            vec![
                evidence(ComparisonKind::SqlCatalog, ComparisonStatus::Pass),
                evidence(ComparisonKind::GeneratedTypes, ComparisonStatus::Pass),
            ],
            inputs(),
        );
        assert_eq!(missing.decision, ParityDecision::Pause);

        let duplicate = PeerAuthorityCertification::evaluate(
            vec![
                evidence(ComparisonKind::SqlCatalog, ComparisonStatus::Pass),
                evidence(ComparisonKind::SqlCatalog, ComparisonStatus::Pass),
                evidence(ComparisonKind::GeneratedTypes, ComparisonStatus::Pass),
                evidence(ComparisonKind::OrmProjection, ComparisonStatus::Pass),
            ],
            inputs(),
        );
        assert_eq!(duplicate.decision, ParityDecision::Pause);
    }

    #[test]
    fn serialized_continue_flag_cannot_override_bad_evidence() {
        let mut certification = PeerAuthorityCertification::evaluate(
            vec![
                evidence(ComparisonKind::SqlCatalog, ComparisonStatus::Pass),
                evidence(ComparisonKind::GeneratedTypes, ComparisonStatus::Pass),
                evidence(ComparisonKind::OrmProjection, ComparisonStatus::Pass),
            ],
            inputs(),
        );
        certification.comparisons[1].status = ComparisonStatus::Error;

        assert!(!certification.is_continue());
    }
}
