//! Fail-closed evaluation of peer contract and ORM comparison evidence.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PEER_AUTHORITY_CERTIFICATION_FORMAT: &str = "declmig.peer-authority-certification/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

/// Exact identities that an all-pass certificate binds.
///
/// Digest fields remain optional so a pause certificate can identify missing
/// artifacts without inventing evidence. A `continue` decision requires every
/// identity to be present, canonical, and lower-case.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PeerAuthorityInputs {
    pub logical_revision: Option<String>,
    pub engine: Option<String>,
    pub engine_version: Option<String>,
    pub desired_catalog_sha256: Option<String>,
    pub dpm_sha256: Option<String>,
    pub type_spec_source_sha256: Option<String>,
    pub json_schema_open_api_source_sha256: Option<String>,
    pub typespec_sql_sha256: Option<String>,
    pub json_schema_open_api_sql_sha256: Option<String>,
    pub protobuf_descriptor_sha256: Option<String>,
    pub grpc_descriptor_sha256: Option<String>,
    pub typespec_types_sha256: Option<String>,
    pub json_schema_open_api_types_sha256: Option<String>,
    pub wire_client_sha256: Option<String>,
    pub write_client_sha256: Option<String>,
    pub sea_orm_projection_sha256: Option<String>,
    pub diesel_projection_sha256: Option<String>,
    pub typespec_generator_sha256: Option<String>,
    pub json_schema_open_api_generator_sha256: Option<String>,
    pub sea_orm_generator_sha256: Option<String>,
    pub diesel_generator_sha256: Option<String>,
}

impl PeerAuthorityInputs {
    fn all_present_and_valid(&self) -> bool {
        [
            self.logical_revision.as_deref(),
            self.engine.as_deref(),
            self.engine_version.as_deref(),
        ]
        .into_iter()
        .all(|value| value.is_some_and(is_canonical_identity))
            && [
                self.desired_catalog_sha256.as_deref(),
                self.dpm_sha256.as_deref(),
                self.type_spec_source_sha256.as_deref(),
                self.json_schema_open_api_source_sha256.as_deref(),
                self.typespec_sql_sha256.as_deref(),
                self.json_schema_open_api_sql_sha256.as_deref(),
                self.protobuf_descriptor_sha256.as_deref(),
                self.grpc_descriptor_sha256.as_deref(),
                self.typespec_types_sha256.as_deref(),
                self.json_schema_open_api_types_sha256.as_deref(),
                self.wire_client_sha256.as_deref(),
                self.write_client_sha256.as_deref(),
                self.sea_orm_projection_sha256.as_deref(),
                self.diesel_projection_sha256.as_deref(),
                self.typespec_generator_sha256.as_deref(),
                self.json_schema_open_api_generator_sha256.as_deref(),
                self.sea_orm_generator_sha256.as_deref(),
                self.diesel_generator_sha256.as_deref(),
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
    pub issued_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub policy: PeerAuthorityPolicy,
    pub comparisons: Vec<ComparisonEvidence>,
    pub inputs: PeerAuthorityInputs,
}

impl PeerAuthorityCertification {
    /// Builds a certificate without selecting a winning authority.
    ///
    /// The serialized decision is `Continue` only when there is exactly one
    /// successful result for every required comparison kind, every evidence and
    /// input identity is canonical, and the certificate has a non-empty validity
    /// window. Consumers still call [`Self::is_continue_for`] at use time.
    #[must_use]
    pub fn evaluate(
        mut comparisons: Vec<ComparisonEvidence>,
        inputs: PeerAuthorityInputs,
        issued_at_unix_seconds: u64,
        expires_at_unix_seconds: u64,
    ) -> Self {
        comparisons.sort_by_key(|item| item.kind);
        let decision_eligible =
            validity_window_is_well_formed(issued_at_unix_seconds, expires_at_unix_seconds)
                && comparison_set_passes(&comparisons)
                && comparisons
                    .iter()
                    .all(|item| item.evidence_sha256.as_deref().is_some_and(is_sha256))
                && inputs.all_present_and_valid();

        Self {
            format: PEER_AUTHORITY_CERTIFICATION_FORMAT.to_owned(),
            decision: if decision_eligible {
                ParityDecision::Continue
            } else {
                ParityDecision::Pause
            },
            decision_eligible,
            issued_at_unix_seconds,
            expires_at_unix_seconds,
            policy: PeerAuthorityPolicy::default(),
            comparisons,
            inputs,
        }
    }

    /// Recomputes release eligibility instead of trusting serialized flags.
    #[must_use]
    pub fn is_continue_at(&self, now_unix_seconds: u64) -> bool {
        self.format == PEER_AUTHORITY_CERTIFICATION_FORMAT
            && self.decision == ParityDecision::Continue
            && self.decision_eligible
            && validity_window_contains(
                self.issued_at_unix_seconds,
                self.expires_at_unix_seconds,
                now_unix_seconds,
            )
            && self.policy == PeerAuthorityPolicy::default()
            && comparison_set_passes(&self.comparisons)
            && self
                .comparisons
                .iter()
                .all(|item| item.evidence_sha256.as_deref().is_some_and(is_sha256))
            && self.inputs.all_present_and_valid()
    }

    /// Requires both current all-pass evidence and exact expected input
    /// identities. This rejects stale certificates and digest substitution.
    #[must_use]
    pub fn is_continue_for(
        &self,
        expected_inputs: &PeerAuthorityInputs,
        now_unix_seconds: u64,
    ) -> bool {
        self.inputs == *expected_inputs && self.is_continue_at(now_unix_seconds)
    }

    /// Returns a deterministic digest of the typed certificate.
    ///
    /// Comparison order is canonicalized before serialization. The digest is a
    /// semantic certificate identity, not a digest of arbitrary JSON whitespace.
    pub fn canonical_sha256(&self) -> Result<String, serde_json::Error> {
        let mut canonical = self.clone();
        canonical.comparisons.sort_by_key(|item| item.kind);
        let encoded = serde_json::to_vec(&canonical)?;
        let digest = Sha256::digest(encoded);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
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
    let all_pass = comparisons.iter().all(|item| {
        item.status == ComparisonStatus::Pass
            && is_canonical_identity(&item.left)
            && is_canonical_identity(&item.right)
    });
    let sql_exit_is_zero = comparisons
        .iter()
        .all(|item| item.kind != ComparisonKind::SqlCatalog || item.tool_exit_code == Some(0));

    unique && complete && all_pass && sql_exit_is_zero
}

fn validity_window_is_well_formed(issued_at: u64, expires_at: u64) -> bool {
    issued_at < expires_at
}

fn validity_window_contains(issued_at: u64, expires_at: u64, now: u64) -> bool {
    validity_window_is_well_formed(issued_at, expires_at) && issued_at <= now && now < expires_at
}

fn is_canonical_identity(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{
        ComparisonEvidence, ComparisonKind, ComparisonStatus, ParityDecision,
        PeerAuthorityCertification, PeerAuthorityInputs,
    };

    const ISSUED_AT: u64 = 100;
    const EXPIRES_AT: u64 = 200;
    const NOW: u64 = 150;

    fn digest(byte: char) -> Option<String> {
        Some(byte.to_string().repeat(64))
    }

    fn evidence(kind: ComparisonKind, status: ComparisonStatus) -> ComparisonEvidence {
        ComparisonEvidence {
            kind,
            left: "left".to_owned(),
            right: "right".to_owned(),
            status,
            tool_exit_code: (kind == ComparisonKind::SqlCatalog).then_some(0),
            message: None,
            evidence_sha256: digest('a'),
        }
    }

    fn passing_comparisons() -> Vec<ComparisonEvidence> {
        vec![
            evidence(ComparisonKind::SqlCatalog, ComparisonStatus::Pass),
            evidence(ComparisonKind::GeneratedTypes, ComparisonStatus::Pass),
            evidence(ComparisonKind::OrmProjection, ComparisonStatus::Pass),
        ]
    }

    fn inputs() -> PeerAuthorityInputs {
        PeerAuthorityInputs {
            logical_revision: Some("contracts-42".to_owned()),
            engine: Some("postgresql".to_owned()),
            engine_version: Some("18.0".to_owned()),
            desired_catalog_sha256: digest('a'),
            dpm_sha256: digest('b'),
            type_spec_source_sha256: digest('c'),
            json_schema_open_api_source_sha256: digest('d'),
            typespec_sql_sha256: digest('e'),
            json_schema_open_api_sql_sha256: digest('f'),
            protobuf_descriptor_sha256: digest('1'),
            grpc_descriptor_sha256: digest('2'),
            typespec_types_sha256: digest('3'),
            json_schema_open_api_types_sha256: digest('4'),
            wire_client_sha256: digest('5'),
            write_client_sha256: digest('6'),
            sea_orm_projection_sha256: digest('7'),
            diesel_projection_sha256: digest('8'),
            typespec_generator_sha256: digest('9'),
            json_schema_open_api_generator_sha256: digest('a'),
            sea_orm_generator_sha256: digest('b'),
            diesel_generator_sha256: digest('c'),
        }
    }

    #[test]
    fn all_three_independent_comparisons_are_required() {
        let certification = PeerAuthorityCertification::evaluate(
            passing_comparisons(),
            inputs(),
            ISSUED_AT,
            EXPIRES_AT,
        );

        assert_eq!(certification.decision, ParityDecision::Continue);
        assert!(certification.is_continue_for(&inputs(), NOW));
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
            ISSUED_AT,
            EXPIRES_AT,
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
            ISSUED_AT,
            EXPIRES_AT,
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
            ISSUED_AT,
            EXPIRES_AT,
        );
        assert_eq!(duplicate.decision, ParityDecision::Pause);
    }

    #[test]
    fn serialized_continue_flag_cannot_override_bad_evidence() {
        let mut certification = PeerAuthorityCertification::evaluate(
            passing_comparisons(),
            inputs(),
            ISSUED_AT,
            EXPIRES_AT,
        );
        certification.comparisons[1].status = ComparisonStatus::Error;

        assert!(!certification.is_continue_for(&inputs(), NOW));
    }

    #[test]
    fn expired_or_not_yet_valid_certificate_pauses_at_use_time() {
        let certification = PeerAuthorityCertification::evaluate(
            passing_comparisons(),
            inputs(),
            ISSUED_AT,
            EXPIRES_AT,
        );

        assert!(!certification.is_continue_for(&inputs(), ISSUED_AT - 1));
        assert!(!certification.is_continue_for(&inputs(), EXPIRES_AT));
    }

    #[test]
    fn input_substitution_and_stale_revision_are_rejected() {
        let certification = PeerAuthorityCertification::evaluate(
            passing_comparisons(),
            inputs(),
            ISSUED_AT,
            EXPIRES_AT,
        );
        let mut substituted = inputs();
        substituted.typespec_sql_sha256 = digest('0');
        assert!(!certification.is_continue_for(&substituted, NOW));

        let mut stale = inputs();
        stale.logical_revision = Some("contracts-43".to_owned());
        assert!(!certification.is_continue_for(&stale, NOW));
    }

    #[test]
    fn noncanonical_uppercase_digest_pauses() {
        let mut invalid = inputs();
        invalid.typespec_sql_sha256 = Some("A".repeat(64));
        let certification = PeerAuthorityCertification::evaluate(
            passing_comparisons(),
            invalid,
            ISSUED_AT,
            EXPIRES_AT,
        );

        assert_eq!(certification.decision, ParityDecision::Pause);
    }

    #[test]
    fn certificate_digest_is_stable_across_comparison_order() {
        let mut reversed = passing_comparisons();
        reversed.reverse();

        let left = PeerAuthorityCertification::evaluate(
            passing_comparisons(),
            inputs(),
            ISSUED_AT,
            EXPIRES_AT,
        );
        let right = PeerAuthorityCertification::evaluate(reversed, inputs(), ISSUED_AT, EXPIRES_AT);

        let left_digest = left.canonical_sha256().expect("serialize certificate");
        let right_digest = right.canonical_sha256().expect("serialize certificate");
        assert_eq!(left_digest, right_digest);
        assert_eq!(left_digest.len(), 64);
        assert!(left_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    }
}
