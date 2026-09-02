import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema("https://github.com/declarative-migrations/declmig-interfaces/schema/v2/")
namespace DeclarativeMigrations.V2;

@minLength(1)
scalar NonEmptyString extends string;

@pattern("^[a-f0-9]{64}$")
scalar Sha256 extends string;

enum DatabaseEngine {
  postgresql,
  cockroachdb,
}

enum SafetyClass {
  safe,
  online,
  blocking,
  destructive,
  manualReview: "manual_review",
}

enum RetryClass {
  never,
  idempotent,
  serializableRestart: "serializable_restart",
  replanRequired: "replan_required",
}

enum RollbackClass {
  reversible,
  compensatable,
  restoreRequired: "restore_required",
  irreversible,
}

enum ApprovalRequirement {
  none,
  operator,
  destructiveOperator: "destructive_operator",
  securityReview: "security_review",
  manual,
}

enum LockExpectation {
  none,
  metadata,
  share,
  exclusive,
  engineManaged: "engine_managed",
}

enum ChangeKind {
  createSchema: "create_schema",
  dropSchema: "drop_schema",
  createTable: "create_table",
  dropTable: "drop_table",
  addColumn: "add_column",
  alterColumn: "alter_column",
  dropColumn: "drop_column",
  addConstraint: "add_constraint",
  dropConstraint: "drop_constraint",
  createIndex: "create_index",
  dropIndex: "drop_index",
  createView: "create_view",
  dropView: "drop_view",
  createFunction: "create_function",
  dropFunction: "drop_function",
  dataBackfill: "data_backfill",
  manual,
}

enum CheckExpectation {
  noRows: "no_rows",
  booleanTrue: "boolean_true",
  scalarEquals: "scalar_equals",
}

model MigrationChange {
  kind: ChangeKind;
  resource: NonEmptyString;
  safety: SafetyClass;
  manual_reason?: NonEmptyString;
}

model PhaseMetadata {
  id: NonEmptyString;
  safety: SafetyClass;
  retry: RetryClass;
  rollback: RollbackClass;
  approval: ApprovalRequirement;
  @minValue(1)
  timeout_seconds: uint32;
  lock_expectation: LockExpectation;
  @minItems(1)
  changes: MigrationChange[];
}

model MigrationStatement {
  @minValue(1)
  ordinal: uint32;
  sql: NonEmptyString;
  sha256: Sha256;
}

model MigrationCheck {
  name: NonEmptyString;
  sql: NonEmptyString;
  expectation: CheckExpectation;
  expected_value?: string;
}

model TransactionalDdlPhase {
  kind: "transactional_ddl";
  metadata: PhaseMetadata;
  @minItems(1)
  statements: MigrationStatement[];
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

model NonTransactionalDdlPhase {
  kind: "non_transactional_ddl";
  metadata: PhaseMetadata;
  @minItems(1)
  statements: MigrationStatement[];
  cleanup_on_failure: NonEmptyString;
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

model CockroachSchemaJobPhase {
  kind: "cockroach_schema_job";
  metadata: PhaseMetadata;
  @minItems(1)
  statements: MigrationStatement[];
  wait_for_terminal_jobs: boolean;
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

model DataBackfillPhase {
  kind: "data_backfill";
  metadata: PhaseMetadata;
  worker_artifact_sha256: Sha256;
  batch_key: NonEmptyString;
  @minValue(1)
  max_batch_size: uint32;
  preconditions: MigrationCheck[];
  @minItems(1)
  postconditions: MigrationCheck[];
}

model ValidationPhase {
  kind: "validation";
  metadata: PhaseMetadata;
  @minItems(1)
  checks: MigrationCheck[];
}

model TrafficTransitionPhase {
  kind: "traffic_transition";
  metadata: PhaseMetadata;
  deployment_gate: NonEmptyString;
}

model DestructiveCleanupPhase {
  kind: "destructive_cleanup";
  metadata: PhaseMetadata;
  @minItems(1)
  statements: MigrationStatement[];
  backup_requirement: NonEmptyString;
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

@oneOf
union MigrationPhase {
  transactionalDdl: TransactionalDdlPhase,
  nonTransactionalDdl: NonTransactionalDdlPhase,
  cockroachSchemaJob: CockroachSchemaJobPhase,
  dataBackfill: DataBackfillPhase,
  validation: ValidationPhase,
  trafficTransition: TrafficTransitionPhase,
  destructiveCleanup: DestructiveCleanupPhase,
}

@id("migrationplan.json")
model MigrationPlan {
  format: "ores.migration-plan/v2";
  id: NonEmptyString;
  revision: NonEmptyString;
  engine: DatabaseEngine;
  engine_version: NonEmptyString;
  source_catalog_sha256: Sha256;
  desired_catalog_sha256: Sha256;
  rendered_sql_sha256: Sha256;
  @minItems(1)
  phases: MigrationPhase[];
}
