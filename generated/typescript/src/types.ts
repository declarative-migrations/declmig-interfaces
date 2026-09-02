export const PROTOCOL_VERSION = "2" as const;
export const SCHEMA_REVISION = "declmig-0002" as const;
export const PLAN_FORMAT = "ores.migration-plan/v2" as const;

export type DatabaseEngine = "postgresql" | "cockroachdb";
export type SafetyClass =
  | "safe"
  | "online"
  | "blocking"
  | "destructive"
  | "manual_review";
export type RetryClass =
  | "never"
  | "idempotent"
  | "serializable_restart"
  | "replan_required";
export type RollbackClass =
  | "reversible"
  | "compensatable"
  | "restore_required"
  | "irreversible";
export type ApprovalRequirement =
  | "none"
  | "operator"
  | "destructive_operator"
  | "security_review"
  | "manual";
export type LockExpectation =
  | "none"
  | "metadata"
  | "share"
  | "exclusive"
  | "engine_managed";
export type ChangeKind =
  | "create_schema"
  | "drop_schema"
  | "create_table"
  | "drop_table"
  | "add_column"
  | "alter_column"
  | "drop_column"
  | "add_constraint"
  | "drop_constraint"
  | "create_index"
  | "drop_index"
  | "create_view"
  | "drop_view"
  | "create_function"
  | "drop_function"
  | "data_backfill"
  | "manual";
export type CheckExpectation =
  | "no_rows"
  | "boolean_true"
  | "scalar_equals";

export interface Health {
  ok: boolean;
  service: string;
  protocol: string;
}

export interface MigrationChange {
  kind: ChangeKind;
  resource: string;
  safety: SafetyClass;
  manual_reason?: string;
}

export interface PhaseMetadata {
  id: string;
  safety: SafetyClass;
  retry: RetryClass;
  rollback: RollbackClass;
  approval: ApprovalRequirement;
  timeout_seconds: number;
  lock_expectation: LockExpectation;
  changes: MigrationChange[];
}

export interface MigrationStatement {
  ordinal: number;
  sql: string;
  sha256: string;
}

export interface MigrationCheck {
  name: string;
  sql: string;
  expectation: CheckExpectation;
  expected_value?: string;
}

export interface TransactionalDdlPhase {
  kind: "transactional_ddl";
  metadata: PhaseMetadata;
  statements: MigrationStatement[];
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

export interface NonTransactionalDdlPhase {
  kind: "non_transactional_ddl";
  metadata: PhaseMetadata;
  statements: MigrationStatement[];
  cleanup_on_failure: string;
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

export interface CockroachSchemaJobPhase {
  kind: "cockroach_schema_job";
  metadata: PhaseMetadata;
  statements: MigrationStatement[];
  wait_for_terminal_jobs: boolean;
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

export interface DataBackfillPhase {
  kind: "data_backfill";
  metadata: PhaseMetadata;
  worker_artifact_sha256: string;
  batch_key: string;
  max_batch_size: number;
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

export interface ValidationPhase {
  kind: "validation";
  metadata: PhaseMetadata;
  checks: MigrationCheck[];
}

export interface TrafficTransitionPhase {
  kind: "traffic_transition";
  metadata: PhaseMetadata;
  deployment_gate: string;
}

export interface DestructiveCleanupPhase {
  kind: "destructive_cleanup";
  metadata: PhaseMetadata;
  statements: MigrationStatement[];
  backup_requirement: string;
  preconditions: MigrationCheck[];
  postconditions: MigrationCheck[];
}

export type MigrationPhase =
  | TransactionalDdlPhase
  | NonTransactionalDdlPhase
  | CockroachSchemaJobPhase
  | DataBackfillPhase
  | ValidationPhase
  | TrafficTransitionPhase
  | DestructiveCleanupPhase;

export interface MigrationPlan {
  format: typeof PLAN_FORMAT;
  id: string;
  revision: string;
  engine: DatabaseEngine;
  engine_version: string;
  source_catalog_sha256: string;
  desired_catalog_sha256: string;
  rendered_sql_sha256: string;
  phases: MigrationPhase[];
}
