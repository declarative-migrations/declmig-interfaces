#![allow(clippy::too_many_lines)]

use declmig_interfaces::migration_plan_v2::{
    ApprovalRequirement, ChangeKind, CheckExpectation, CockroachSchemaJobPhase,
    DataBackfillPhase, DatabaseEngine, DestructiveCleanupPhase, LockExpectation,
    MigrationChange, MigrationCheck, MigrationPhase, MigrationStatement,
    NonTransactionalDdlPhase, PhaseMetadata, RetryClass, RollbackClass, SafetyClass,
    TrafficTransitionPhase, TransactionalDdlPhase, ValidationPhase,
};
use declmig_interfaces::{MigrationPlanV2, MigrationPlanV2Error, MIGRATION_PLAN_FORMAT};
use serde_json::Value;

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn change(kind: ChangeKind, resource: &str, safety: SafetyClass) -> MigrationChange {
    MigrationChange {
        kind,
        resource: resource.to_owned(),
        safety,
        manual_reason: (safety == SafetyClass::ManualReview)
            .then(|| "explicit operator transition".to_owned()),
    }
}

fn metadata(
    id: &str,
    safety: SafetyClass,
    approval: ApprovalRequirement,
    change: MigrationChange,
) -> PhaseMetadata {
    PhaseMetadata {
        id: id.to_owned(),
        safety,
        retry: RetryClass::Idempotent,
        rollback: RollbackClass::Reversible,
        approval,
        timeout_seconds: 60,
        lock_expectation: LockExpectation::Metadata,
        changes: vec![change],
    }
}

fn statement(ordinal: u32, sql: &str, digest_character: char) -> MigrationStatement {
    MigrationStatement {
        ordinal,
        sql: sql.to_owned(),
        sha256: digest(digest_character),
    }
}

fn check(expectation: CheckExpectation, expected_value: Option<&str>) -> MigrationCheck {
    MigrationCheck {
        name: "catalog invariant".to_owned(),
        sql: "SELECT true".to_owned(),
        expectation,
        expected_value: expected_value.map(str::to_owned),
    }
}

fn all_phase_plan() -> MigrationPlanV2 {
    MigrationPlanV2 {
        format: MIGRATION_PLAN_FORMAT.to_owned(),
        id: "all-phases".to_owned(),
        revision: "declmig-0002".to_owned(),
        engine: DatabaseEngine::Cockroachdb,
        engine_version: "26.2.0".to_owned(),
        source_catalog_sha256: digest('a'),
        desired_catalog_sha256: digest('b'),
        rendered_sql_sha256: digest('c'),
        phases: vec![
            MigrationPhase::TransactionalDdl(TransactionalDdlPhase {
                metadata: metadata(
                    "transactional",
                    SafetyClass::Safe,
                    ApprovalRequirement::None,
                    change(
                        ChangeKind::CreateTable,
                        "public.accounts",
                        SafetyClass::Safe,
                    ),
                ),
                statements: vec![statement(
                    1,
                    "CREATE TABLE public.accounts (id bigint PRIMARY KEY)",
                    'd',
                )],
                preconditions: vec![],
                postconditions: vec![],
            }),
            MigrationPhase::NonTransactionalDdl(NonTransactionalDdlPhase {
                metadata: metadata(
                    "non-transactional",
                    SafetyClass::Online,
                    ApprovalRequirement::Operator,
                    change(
                        ChangeKind::CreateIndex,
                        "public.accounts_id_idx",
                        SafetyClass::Online,
                    ),
                ),
                statements: vec![statement(
                    1,
                    "CREATE INDEX CONCURRENTLY accounts_id_idx ON public.accounts (id)",
                    'e',
                )],
                cleanup_on_failure: "DROP INDEX CONCURRENTLY IF EXISTS accounts_id_idx"
                    .to_owned(),
                preconditions: vec![],
                postconditions: vec![check(CheckExpectation::BooleanTrue, None)],
            }),
            MigrationPhase::CockroachSchemaJob(CockroachSchemaJobPhase {
                metadata: metadata(
                    "cockroach-job",
                    SafetyClass::Online,
                    ApprovalRequirement::Operator,
                    change(
                        ChangeKind::AddColumn,
                        "public.accounts.display_name",
                        SafetyClass::Online,
                    ),
                ),
                statements: vec![statement(
                    1,
                    "ALTER TABLE public.accounts ADD COLUMN display_name string",
                    'f',
                )],
                wait_for_terminal_jobs: true,
                preconditions: vec![],
                postconditions: vec![check(CheckExpectation::NoRows, None)],
            }),
            MigrationPhase::DataBackfill(DataBackfillPhase {
                metadata: metadata(
                    "backfill",
                    SafetyClass::Online,
                    ApprovalRequirement::Operator,
                    change(
                        ChangeKind::DataBackfill,
                        "public.accounts.display_name",
                        SafetyClass::Online,
                    ),
                ),
                worker_artifact_sha256: digest('1'),
                batch_key: "accounts.id".to_owned(),
                max_batch_size: 500,
                preconditions: vec![],
                postconditions: vec![check(CheckExpectation::NoRows, None)],
            }),
            MigrationPhase::Validation(ValidationPhase {
                metadata: metadata(
                    "validation",
                    SafetyClass::Safe,
                    ApprovalRequirement::None,
                    change(
                        ChangeKind::AddConstraint,
                        "public.accounts_display_name_check",
                        SafetyClass::Safe,
                    ),
                ),
                checks: vec![check(CheckExpectation::ScalarEquals, Some("0"))],
            }),
            MigrationPhase::TrafficTransition(TrafficTransitionPhase {
                metadata: metadata(
                    "traffic",
                    SafetyClass::ManualReview,
                    ApprovalRequirement::Manual,
                    change(
                        ChangeKind::Manual,
                        "deployment/accounts-v2",
                        SafetyClass::ManualReview,
                    ),
                ),
                deployment_gate: "accounts-v2-read-write-canary".to_owned(),
            }),
            MigrationPhase::DestructiveCleanup(DestructiveCleanupPhase {
                metadata: metadata(
                    "cleanup",
                    SafetyClass::Destructive,
                    ApprovalRequirement::DestructiveOperator,
                    change(
                        ChangeKind::DropColumn,
                        "public.accounts.legacy_name",
                        SafetyClass::Destructive,
                    ),
                ),
                statements: vec![statement(
                    1,
                    "ALTER TABLE public.accounts DROP COLUMN legacy_name",
                    '2',
                )],
                backup_requirement: "pitr-and-verified-logical-export".to_owned(),
                preconditions: vec![check(CheckExpectation::NoRows, None)],
                postconditions: vec![check(CheckExpectation::BooleanTrue, None)],
            }),
        ],
    }
}

#[test]
fn round_trips_every_phase_variant_with_stable_tags() {
    let expected = all_phase_plan();
    expected.validate().expect("all phase variants must validate");

    let json = serde_json::to_string(&expected).expect("serialize all-phase plan");
    let actual = MigrationPlanV2::from_json(&json).expect("parse all-phase plan");
    assert_eq!(actual, expected);

    let value: Value = serde_json::from_str(&json).expect("decode JSON value");
    let tags = value["phases"]
        .as_array()
        .expect("phases array")
        .iter()
        .map(|phase| phase["kind"].as_str().expect("phase kind"))
        .collect::<Vec<_>>();
    assert_eq!(
        tags,
        [
            "transactional_ddl",
            "non_transactional_ddl",
            "cockroach_schema_job",
            "data_backfill",
            "validation",
            "traffic_transition",
            "destructive_cleanup",
        ]
    );
}

#[test]
fn rejects_nested_unknown_fields_and_unknown_phase_kinds() {
    let mut value = serde_json::to_value(all_phase_plan()).expect("encode fixture");
    value["phases"][0]["metadata"]["unexpected"] = Value::Bool(true);
    assert_eq!(
        MigrationPlanV2::from_json(&value.to_string()),
        Err(MigrationPlanV2Error::SchemaMismatch)
    );

    let mut value = serde_json::to_value(all_phase_plan()).expect("encode fixture");
    value["phases"][0]["kind"] = Value::String("future_phase".to_owned());
    assert_eq!(
        MigrationPlanV2::from_json(&value.to_string()),
        Err(MigrationPlanV2Error::SchemaMismatch)
    );
}

#[test]
fn rejects_statement_and_check_ambiguity() {
    let mut plan = all_phase_plan();
    let MigrationPhase::TransactionalDdl(phase) = &mut plan.phases[0] else {
        panic!("fixture phase order changed");
    };
    phase.statements[0].ordinal = 2;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "statement ordinals must be contiguous and start at one"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::Validation(phase) = &mut plan.phases[4] else {
        panic!("fixture phase order changed");
    };
    phase.checks[0].expected_value = None;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "scalar_equals checks require expected_value"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::Validation(phase) = &mut plan.phases[4] else {
        panic!("fixture phase order changed");
    };
    phase.checks[0].expectation = CheckExpectation::NoRows;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "expected_value is allowed only for scalar_equals checks"
        ))
    );
}

#[test]
fn rejects_backfill_validation_traffic_and_cleanup_ambiguity() {
    let mut plan = all_phase_plan();
    let MigrationPhase::DataBackfill(phase) = &mut plan.phases[3] else {
        panic!("fixture phase order changed");
    };
    phase.max_batch_size = 0;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "max_batch_size must be greater than zero"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::DataBackfill(phase) = &mut plan.phases[3] else {
        panic!("fixture phase order changed");
    };
    phase.postconditions.clear();
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "data_backfill requires at least one postcondition"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::Validation(phase) = &mut plan.phases[4] else {
        panic!("fixture phase order changed");
    };
    phase.checks.clear();
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "validation requires at least one check"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::TrafficTransition(phase) = &mut plan.phases[5] else {
        panic!("fixture phase order changed");
    };
    phase.deployment_gate = "  ".to_owned();
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::EmptyField("deployment_gate"))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::DestructiveCleanup(phase) = &mut plan.phases[6] else {
        panic!("fixture phase order changed");
    };
    phase.metadata.safety = SafetyClass::Online;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "destructive_cleanup requires destructive safety"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::DestructiveCleanup(phase) = &mut plan.phases[6] else {
        panic!("fixture phase order changed");
    };
    phase.metadata.approval = ApprovalRequirement::Operator;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "destructive_cleanup requires destructive_operator approval"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::DestructiveCleanup(phase) = &mut plan.phases[6] else {
        panic!("fixture phase order changed");
    };
    phase.backup_requirement = "".to_owned();
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::EmptyField("backup_requirement"))
    );
}

#[test]
fn rejects_plan_phase_and_manual_review_identity_errors() {
    let mut plan = all_phase_plan();
    plan.format = "ores.migration-plan/v3".to_owned();
    assert_eq!(plan.validate(), Err(MigrationPlanV2Error::InvalidFormat));

    let mut plan = all_phase_plan();
    plan.id = " ".to_owned();
    assert_eq!(plan.validate(), Err(MigrationPlanV2Error::EmptyField("id")));

    let mut plan = all_phase_plan();
    plan.rendered_sql_sha256 = digest('A');
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidSha256(
            "rendered_sql_sha256"
        ))
    );

    let mut plan = all_phase_plan();
    plan.phases.clear();
    assert_eq!(plan.validate(), Err(MigrationPlanV2Error::EmptyPhases));

    let mut plan = all_phase_plan();
    plan.phases.push(plan.phases[0].clone());
    assert_eq!(plan.validate(), Err(MigrationPlanV2Error::DuplicatePhaseId));

    let mut plan = all_phase_plan();
    plan.engine = DatabaseEngine::Postgresql;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "cockroach_schema_job requires engine=cockroachdb"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::TrafficTransition(phase) = &mut plan.phases[5] else {
        panic!("fixture phase order changed");
    };
    phase.metadata.changes[0].manual_reason = None;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "manual_review changes require manual_reason"
        ))
    );

    let mut plan = all_phase_plan();
    let MigrationPhase::TransactionalDdl(phase) = &mut plan.phases[0] else {
        panic!("fixture phase order changed");
    };
    phase.metadata.changes[0].safety = SafetyClass::Destructive;
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "destructive changes must use destructive_cleanup"
        ))
    );
}
