#![forbid(unsafe_code)]

//! Strict, typed migration-plan v2 data contract.
//!
//! This module preserves the useful typed phase model from the superseded v2
//! branch without replacing the stable v1 protocol. It is a Rust validation
//! surface only: publication remains blocked until independently authored
//! TypeSpec and JSON Schema/OpenAPI roots generate semantically equal plan
//! contracts and client projections.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Protocol revision carried by typed migration-plan v2 consumers.
pub const MIGRATION_PLAN_PROTOCOL_VERSION: &str = "2";

/// Stable discriminator for the typed migration-plan envelope.
pub const MIGRATION_PLAN_FORMAT: &str = "ores.migration-plan/v2";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseEngine {
    Postgresql,
    Cockroachdb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyClass {
    Safe,
    Online,
    Blocking,
    Destructive,
    ManualReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryClass {
    Never,
    Idempotent,
    SerializableRestart,
    ReplanRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackClass {
    Reversible,
    Compensatable,
    RestoreRequired,
    Irreversible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRequirement {
    None,
    Operator,
    DestructiveOperator,
    SecurityReview,
    Manual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockExpectation {
    None,
    Metadata,
    Share,
    Exclusive,
    EngineManaged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    CreateSchema,
    DropSchema,
    CreateTable,
    DropTable,
    AddColumn,
    AlterColumn,
    DropColumn,
    AddConstraint,
    DropConstraint,
    CreateIndex,
    DropIndex,
    CreateView,
    DropView,
    CreateFunction,
    DropFunction,
    DataBackfill,
    Manual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckExpectation {
    NoRows,
    BooleanTrue,
    ScalarEquals,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationChange {
    pub kind: ChangeKind,
    pub resource: String,
    pub safety: SafetyClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseMetadata {
    pub id: String,
    pub safety: SafetyClass,
    pub retry: RetryClass,
    pub rollback: RollbackClass,
    pub approval: ApprovalRequirement,
    pub timeout_seconds: u32,
    pub lock_expectation: LockExpectation,
    pub changes: Vec<MigrationChange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationStatement {
    pub ordinal: u32,
    pub sql: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationCheck {
    pub name: String,
    pub sql: String,
    pub expectation: CheckExpectation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransactionalDdlPhase {
    pub metadata: PhaseMetadata,
    pub statements: Vec<MigrationStatement>,
    pub preconditions: Vec<MigrationCheck>,
    pub postconditions: Vec<MigrationCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NonTransactionalDdlPhase {
    pub metadata: PhaseMetadata,
    pub statements: Vec<MigrationStatement>,
    pub cleanup_on_failure: String,
    pub preconditions: Vec<MigrationCheck>,
    pub postconditions: Vec<MigrationCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CockroachSchemaJobPhase {
    pub metadata: PhaseMetadata,
    pub statements: Vec<MigrationStatement>,
    pub wait_for_terminal_jobs: bool,
    pub preconditions: Vec<MigrationCheck>,
    pub postconditions: Vec<MigrationCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataBackfillPhase {
    pub metadata: PhaseMetadata,
    pub worker_artifact_sha256: String,
    pub batch_key: String,
    pub max_batch_size: u32,
    pub preconditions: Vec<MigrationCheck>,
    pub postconditions: Vec<MigrationCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationPhase {
    pub metadata: PhaseMetadata,
    pub checks: Vec<MigrationCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrafficTransitionPhase {
    pub metadata: PhaseMetadata,
    pub deployment_gate: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DestructiveCleanupPhase {
    pub metadata: PhaseMetadata,
    pub statements: Vec<MigrationStatement>,
    pub backup_requirement: String,
    pub preconditions: Vec<MigrationCheck>,
    pub postconditions: Vec<MigrationCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MigrationPhase {
    TransactionalDdl(TransactionalDdlPhase),
    NonTransactionalDdl(NonTransactionalDdlPhase),
    CockroachSchemaJob(CockroachSchemaJobPhase),
    DataBackfill(DataBackfillPhase),
    Validation(ValidationPhase),
    TrafficTransition(TrafficTransitionPhase),
    DestructiveCleanup(DestructiveCleanupPhase),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationPlanV2 {
    pub format: String,
    pub id: String,
    pub revision: String,
    pub engine: DatabaseEngine,
    pub engine_version: String,
    pub source_catalog_sha256: String,
    pub desired_catalog_sha256: String,
    pub rendered_sql_sha256: String,
    pub phases: Vec<MigrationPhase>,
}

impl MigrationPlanV2 {
    /// Decodes and validates a typed migration-plan v2 envelope.
    ///
    /// # Errors
    ///
    /// Returns [`MigrationPlanV2Error::SchemaMismatch`] when decoding fails or
    /// a field-specific error when cross-field invariants are violated.
    pub fn from_json(input: &str) -> Result<Self, MigrationPlanV2Error> {
        let plan: Self =
            serde_json::from_str(input).map_err(|_| MigrationPlanV2Error::SchemaMismatch)?;
        plan.validate()?;
        Ok(plan)
    }

    /// Validates invariants that serialization shape alone cannot express.
    ///
    /// # Errors
    ///
    /// Returns an error when the plan is ambiguous, unsafe, or internally
    /// inconsistent.
    pub fn validate(&self) -> Result<(), MigrationPlanV2Error> {
        if self.format != MIGRATION_PLAN_FORMAT {
            return Err(MigrationPlanV2Error::InvalidFormat);
        }
        require_non_empty(&self.id, "id")?;
        require_non_empty(&self.revision, "revision")?;
        require_non_empty(&self.engine_version, "engine_version")?;
        require_sha256(&self.source_catalog_sha256, "source_catalog_sha256")?;
        require_sha256(&self.desired_catalog_sha256, "desired_catalog_sha256")?;
        require_sha256(&self.rendered_sql_sha256, "rendered_sql_sha256")?;
        if self.phases.is_empty() {
            return Err(MigrationPlanV2Error::EmptyPhases);
        }

        let mut phase_ids = BTreeSet::new();
        for phase in &self.phases {
            let metadata = phase.metadata();
            validate_metadata(metadata, phase.is_destructive_cleanup())?;
            if !phase_ids.insert(metadata.id.as_str()) {
                return Err(MigrationPlanV2Error::DuplicatePhaseId);
            }
            phase.validate(self.engine)?;
        }
        Ok(())
    }
}

impl MigrationPhase {
    fn metadata(&self) -> &PhaseMetadata {
        match self {
            Self::TransactionalDdl(phase) => &phase.metadata,
            Self::NonTransactionalDdl(phase) => &phase.metadata,
            Self::CockroachSchemaJob(phase) => &phase.metadata,
            Self::DataBackfill(phase) => &phase.metadata,
            Self::Validation(phase) => &phase.metadata,
            Self::TrafficTransition(phase) => &phase.metadata,
            Self::DestructiveCleanup(phase) => &phase.metadata,
        }
    }

    fn is_destructive_cleanup(&self) -> bool {
        matches!(self, Self::DestructiveCleanup(_))
    }

    fn validate(&self, engine: DatabaseEngine) -> Result<(), MigrationPlanV2Error> {
        match self {
            Self::TransactionalDdl(phase) => {
                validate_statements(&phase.statements)?;
                validate_checks(&phase.preconditions)?;
                validate_checks(&phase.postconditions)
            }
            Self::NonTransactionalDdl(phase) => {
                validate_statements(&phase.statements)?;
                require_non_empty(&phase.cleanup_on_failure, "cleanup_on_failure")?;
                validate_checks(&phase.preconditions)?;
                validate_checks(&phase.postconditions)
            }
            Self::CockroachSchemaJob(phase) => {
                if engine != DatabaseEngine::Cockroachdb {
                    return Err(MigrationPlanV2Error::InvalidPhase(
                        "cockroach_schema_job requires engine=cockroachdb",
                    ));
                }
                validate_statements(&phase.statements)?;
                validate_checks(&phase.preconditions)?;
                validate_checks(&phase.postconditions)
            }
            Self::DataBackfill(phase) => {
                require_sha256(&phase.worker_artifact_sha256, "worker_artifact_sha256")?;
                require_non_empty(&phase.batch_key, "batch_key")?;
                if phase.max_batch_size == 0 {
                    return Err(MigrationPlanV2Error::InvalidPhase(
                        "max_batch_size must be greater than zero",
                    ));
                }
                validate_checks(&phase.preconditions)?;
                if phase.postconditions.is_empty() {
                    return Err(MigrationPlanV2Error::InvalidPhase(
                        "data_backfill requires at least one postcondition",
                    ));
                }
                validate_checks(&phase.postconditions)
            }
            Self::Validation(phase) => {
                if phase.checks.is_empty() {
                    return Err(MigrationPlanV2Error::InvalidPhase(
                        "validation requires at least one check",
                    ));
                }
                validate_checks(&phase.checks)
            }
            Self::TrafficTransition(phase) => {
                require_non_empty(&phase.deployment_gate, "deployment_gate")
            }
            Self::DestructiveCleanup(phase) => {
                if phase.metadata.safety != SafetyClass::Destructive {
                    return Err(MigrationPlanV2Error::InvalidPhase(
                        "destructive_cleanup requires destructive safety",
                    ));
                }
                if phase.metadata.approval != ApprovalRequirement::DestructiveOperator {
                    return Err(MigrationPlanV2Error::InvalidPhase(
                        "destructive_cleanup requires destructive_operator approval",
                    ));
                }
                require_non_empty(&phase.backup_requirement, "backup_requirement")?;
                validate_statements(&phase.statements)?;
                validate_checks(&phase.preconditions)?;
                validate_checks(&phase.postconditions)
            }
        }
    }
}

fn validate_metadata(
    metadata: &PhaseMetadata,
    destructive_cleanup: bool,
) -> Result<(), MigrationPlanV2Error> {
    require_non_empty(&metadata.id, "phase.metadata.id")?;
    if metadata.timeout_seconds == 0 {
        return Err(MigrationPlanV2Error::InvalidPhase(
            "timeout_seconds must be greater than zero",
        ));
    }
    if metadata.changes.is_empty() {
        return Err(MigrationPlanV2Error::InvalidPhase(
            "phase metadata requires at least one change",
        ));
    }

    for change in &metadata.changes {
        require_non_empty(&change.resource, "change.resource")?;
        if change.safety == SafetyClass::ManualReview
            && change
                .manual_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
        {
            return Err(MigrationPlanV2Error::InvalidPhase(
                "manual_review changes require manual_reason",
            ));
        }
        if change.safety == SafetyClass::Destructive && !destructive_cleanup {
            return Err(MigrationPlanV2Error::InvalidPhase(
                "destructive changes must use destructive_cleanup",
            ));
        }
    }
    Ok(())
}

fn validate_statements(statements: &[MigrationStatement]) -> Result<(), MigrationPlanV2Error> {
    if statements.is_empty() {
        return Err(MigrationPlanV2Error::InvalidPhase(
            "DDL phase requires at least one statement",
        ));
    }
    for (index, statement) in statements.iter().enumerate() {
        let expected = u32::try_from(index + 1)
            .map_err(|_| MigrationPlanV2Error::InvalidPhase("statement count exceeds u32"))?;
        if statement.ordinal != expected {
            return Err(MigrationPlanV2Error::InvalidPhase(
                "statement ordinals must be contiguous and start at one",
            ));
        }
        require_non_empty(&statement.sql, "statement.sql")?;
        require_sha256(&statement.sha256, "statement.sha256")?;
        if statement.sha256 != sha256_hex(&statement.sql) {
            return Err(MigrationPlanV2Error::InvalidPhase(
                "statement.sha256 must match statement.sql",
            ));
        }
    }
    Ok(())
}

fn validate_checks(checks: &[MigrationCheck]) -> Result<(), MigrationPlanV2Error> {
    for check in checks {
        require_non_empty(&check.name, "check.name")?;
        require_non_empty(&check.sql, "check.sql")?;
        if check.expectation == CheckExpectation::ScalarEquals && check.expected_value.is_none() {
            return Err(MigrationPlanV2Error::InvalidPhase(
                "scalar_equals checks require expected_value",
            ));
        }
        if check.expectation != CheckExpectation::ScalarEquals && check.expected_value.is_some() {
            return Err(MigrationPlanV2Error::InvalidPhase(
                "expected_value is allowed only for scalar_equals checks",
            ));
        }
    }
    Ok(())
}

fn sha256_hex(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), MigrationPlanV2Error> {
    if value.trim().is_empty() {
        Err(MigrationPlanV2Error::EmptyField(field))
    } else {
        Ok(())
    }
}

fn require_sha256(value: &str, field: &'static str) -> Result<(), MigrationPlanV2Error> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(MigrationPlanV2Error::InvalidSha256(field))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MigrationPlanV2Error {
    InvalidFormat,
    EmptyField(&'static str),
    InvalidSha256(&'static str),
    EmptyPhases,
    DuplicatePhaseId,
    InvalidPhase(&'static str),
    SchemaMismatch,
}

impl fmt::Display for MigrationPlanV2Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => formatter.write_str("migration plan format is unsupported"),
            Self::EmptyField(field) => write!(formatter, "{field} must be non-empty"),
            Self::InvalidSha256(field) => {
                write!(
                    formatter,
                    "{field} must be a lowercase hexadecimal SHA-256 digest"
                )
            }
            Self::EmptyPhases => {
                formatter.write_str("migration plan must contain at least one phase")
            }
            Self::DuplicatePhaseId => formatter.write_str("migration phase ids must be unique"),
            Self::InvalidPhase(reason) => {
                write!(formatter, "migration phase is invalid: {reason}")
            }
            Self::SchemaMismatch => formatter.write_str("payload does not match migration-plan v2"),
        }
    }
}

impl std::error::Error for MigrationPlanV2Error {}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    fn metadata(id: &str) -> PhaseMetadata {
        PhaseMetadata {
            id: id.to_owned(),
            safety: SafetyClass::Online,
            retry: RetryClass::Idempotent,
            rollback: RollbackClass::Reversible,
            approval: ApprovalRequirement::Operator,
            timeout_seconds: 30,
            lock_expectation: LockExpectation::Metadata,
            changes: vec![MigrationChange {
                kind: ChangeKind::AddColumn,
                resource: "public.accounts.display_name".to_owned(),
                safety: SafetyClass::Online,
                manual_reason: None,
            }],
        }
    }

    fn statement() -> MigrationStatement {
        let sql = "ALTER TABLE public.accounts ADD COLUMN display_name text";
        MigrationStatement {
            ordinal: 1,
            sql: sql.to_owned(),
            sha256: sha256_hex(sql),
        }
    }

    fn valid_plan() -> MigrationPlanV2 {
        MigrationPlanV2 {
            format: MIGRATION_PLAN_FORMAT.to_owned(),
            id: "plan-42".to_owned(),
            revision: "declmig-0002".to_owned(),
            engine: DatabaseEngine::Postgresql,
            engine_version: "17.6".to_owned(),
            source_catalog_sha256: digest('a'),
            desired_catalog_sha256: digest('b'),
            rendered_sql_sha256: digest('c'),
            phases: vec![MigrationPhase::TransactionalDdl(TransactionalDdlPhase {
                metadata: metadata("expand"),
                statements: vec![statement()],
                preconditions: vec![],
                postconditions: vec![],
            })],
        }
    }

    #[test]
    fn round_trips_a_typed_plan() {
        let expected = valid_plan();
        let json = serde_json::to_string(&expected).expect("serialize typed plan");
        let actual = MigrationPlanV2::from_json(&json).expect("parse typed plan");
        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_the_legacy_opaque_payload_shape() {
        let legacy = r#"{"id":"p","revision":"r","payload":{}}"#;
        assert_eq!(
            MigrationPlanV2::from_json(legacy),
            Err(MigrationPlanV2Error::SchemaMismatch)
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let mut value = serde_json::to_value(valid_plan()).expect("encode fixture");
        value
            .as_object_mut()
            .expect("plan object")
            .insert("unknown".to_owned(), serde_json::Value::Bool(true));
        assert_eq!(
            MigrationPlanV2::from_json(&value.to_string()),
            Err(MigrationPlanV2Error::SchemaMismatch)
        );
    }

    #[test]
    fn rejects_duplicate_phase_ids() {
        let mut plan = valid_plan();
        plan.phases.push(plan.phases[0].clone());
        assert_eq!(plan.validate(), Err(MigrationPlanV2Error::DuplicatePhaseId));
    }

    #[test]
    fn rejects_destructive_changes_outside_cleanup() {
        let mut plan = valid_plan();
        let MigrationPhase::TransactionalDdl(phase) = &mut plan.phases[0] else {
            panic!("test fixture changed");
        };
        phase.metadata.changes[0].safety = SafetyClass::Destructive;
        assert_eq!(
            plan.validate(),
            Err(MigrationPlanV2Error::InvalidPhase(
                "destructive changes must use destructive_cleanup"
            ))
        );
    }

    #[test]
    fn rejects_statement_digest_that_does_not_match_sql() {
        let mut plan = valid_plan();
        let MigrationPhase::TransactionalDdl(phase) = &mut plan.phases[0] else {
            panic!("test fixture changed");
        };
        phase.statements[0].sql.push_str(" -- tampered");
        assert_eq!(
            plan.validate(),
            Err(MigrationPlanV2Error::InvalidPhase(
                "statement.sha256 must match statement.sql"
            ))
        );
    }

    #[test]
    fn rejects_non_lowercase_sha256() {
        let mut plan = valid_plan();
        plan.source_catalog_sha256 = digest('A');
        assert_eq!(
            plan.validate(),
            Err(MigrationPlanV2Error::InvalidSha256("source_catalog_sha256"))
        );
    }

    #[test]
    fn rejects_cockroach_phase_for_postgresql_plan() {
        let mut plan = valid_plan();
        plan.phases = vec![MigrationPhase::CockroachSchemaJob(
            CockroachSchemaJobPhase {
                metadata: metadata("schema-job"),
                statements: vec![statement()],
                wait_for_terminal_jobs: true,
                preconditions: vec![],
                postconditions: vec![],
            },
        )];
        assert_eq!(
            plan.validate(),
            Err(MigrationPlanV2Error::InvalidPhase(
                "cockroach_schema_job requires engine=cockroachdb"
            ))
        );
    }
}
