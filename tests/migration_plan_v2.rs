use declmig_interfaces::{
    migration_plan_v2::{
        ApprovalRequirement, ChangeKind, DatabaseEngine, LockExpectation, MigrationChange,
        MigrationPhase, MigrationStatement, PhaseMetadata, RetryClass, RollbackClass, SafetyClass,
        TransactionalDdlPhase,
    },
    MigrationPlan, MigrationPlanV2, MigrationPlanV2Error, MIGRATION_PLAN_FORMAT,
    MIGRATION_PLAN_PROTOCOL_VERSION, PROTOCOL_VERSION,
};
use serde_json::json;
use sha2::{Digest, Sha256};

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn statement_sha256(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}

fn typed_plan() -> MigrationPlanV2 {
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
            metadata: PhaseMetadata {
                id: "expand".to_owned(),
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
            },
            statements: vec![MigrationStatement {
                ordinal: 1,
                sql: "ALTER TABLE public.accounts ADD COLUMN display_name text".to_owned(),
                sha256: statement_sha256(
                    "ALTER TABLE public.accounts ADD COLUMN display_name text",
                ),
            }],
            preconditions: vec![],
            postconditions: vec![],
        })],
    }
}

#[test]
fn v1_and_v2_protocols_coexist_without_reinterpreting_payloads() {
    assert_eq!(PROTOCOL_VERSION, "1");
    assert_eq!(MIGRATION_PLAN_PROTOCOL_VERSION, "2");

    let legacy = MigrationPlan::parse("legacy".to_owned(), "r1".to_owned(), json!({}))
        .expect("v1 remains available");
    let legacy_json = serde_json::to_string(&legacy).expect("encode v1");
    assert_eq!(
        MigrationPlanV2::from_json(&legacy_json),
        Err(MigrationPlanV2Error::SchemaMismatch)
    );

    let v2 = typed_plan();
    let v2_json = serde_json::to_string(&v2).expect("encode v2");
    assert_eq!(MigrationPlanV2::from_json(&v2_json), Ok(v2));
}

#[test]
fn public_v2_contract_fails_closed_on_unsafe_or_ambiguous_input() {
    let mut plan = typed_plan();
    plan.phases.push(plan.phases[0].clone());
    assert_eq!(plan.validate(), Err(MigrationPlanV2Error::DuplicatePhaseId));

    let mut plan = typed_plan();
    let MigrationPhase::TransactionalDdl(phase) = &mut plan.phases[0] else {
        panic!("test fixture changed");
    };
    phase.statements[0].sql.push_str(" -- modified");
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidPhase(
            "statement.sha256 must match statement.sql"
        ))
    );

    let mut plan = typed_plan();
    plan.rendered_sql_sha256 = "not-a-digest".to_owned();
    assert_eq!(
        plan.validate(),
        Err(MigrationPlanV2Error::InvalidSha256("rendered_sql_sha256"))
    );
}
