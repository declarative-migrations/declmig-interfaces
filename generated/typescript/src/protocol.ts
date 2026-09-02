import { InterfaceError } from "./errors.js";
import {
  PLAN_FORMAT,
  type MigrationCheck,
  type MigrationPhase,
  type MigrationPlan,
  type MigrationStatement,
  type PhaseMetadata,
} from "./types.js";

const SHA256 = /^[a-f0-9]{64}$/;

export function validateMigrationPlan(plan: MigrationPlan): MigrationPlan {
  if (plan.format !== PLAN_FORMAT) {
    throw new InterfaceError("invalid_format", "unsupported migration-plan format");
  }
  requireText(plan.id, "id");
  requireText(plan.revision, "revision");
  requireText(plan.engine_version, "engine_version");
  requireSha256(plan.source_catalog_sha256, "source_catalog_sha256");
  requireSha256(plan.desired_catalog_sha256, "desired_catalog_sha256");
  requireSha256(plan.rendered_sql_sha256, "rendered_sql_sha256");
  if (plan.phases.length === 0) {
    throw new InterfaceError("empty_phases", "at least one phase is required");
  }

  const phaseIds = new Set<string>();
  for (const phase of plan.phases) {
    validateMetadata(phase.metadata, phase.kind === "destructive_cleanup");
    if (phaseIds.has(phase.metadata.id)) {
      throw new InterfaceError("duplicate_phase_id", "phase ids must be unique");
    }
    phaseIds.add(phase.metadata.id);
    validatePhase(phase, plan.engine);
  }
  return plan;
}

function validateMetadata(
  metadata: PhaseMetadata,
  destructiveCleanup: boolean,
): void {
  requireText(metadata.id, "phase.metadata.id");
  if (!Number.isInteger(metadata.timeout_seconds) || metadata.timeout_seconds < 1) {
    invalidPhase("timeout_seconds must be a positive integer");
  }
  if (metadata.changes.length === 0) {
    invalidPhase("phase metadata requires at least one change");
  }
  for (const change of metadata.changes) {
    requireText(change.resource, "change.resource");
    if (
      change.safety === "manual_review" &&
      (!change.manual_reason || !change.manual_reason.trim())
    ) {
      invalidPhase("manual_review changes require manual_reason");
    }
    if (change.safety === "destructive" && !destructiveCleanup) {
      invalidPhase("destructive changes must use destructive_cleanup");
    }
  }
}

function validatePhase(phase: MigrationPhase, engine: string): void {
  switch (phase.kind) {
    case "transactional_ddl":
      validateStatements(phase.statements);
      validateChecks(phase.preconditions);
      validateChecks(phase.postconditions);
      return;
    case "non_transactional_ddl":
      validateStatements(phase.statements);
      requireText(phase.cleanup_on_failure, "cleanup_on_failure");
      validateChecks(phase.preconditions);
      validateChecks(phase.postconditions);
      return;
    case "cockroach_schema_job":
      if (engine !== "cockroachdb") {
        invalidPhase("cockroach_schema_job requires engine=cockroachdb");
      }
      validateStatements(phase.statements);
      validateChecks(phase.preconditions);
      validateChecks(phase.postconditions);
      return;
    case "data_backfill":
      requireSha256(phase.worker_artifact_sha256, "worker_artifact_sha256");
      requireText(phase.batch_key, "batch_key");
      if (!Number.isInteger(phase.max_batch_size) || phase.max_batch_size < 1) {
        invalidPhase("max_batch_size must be a positive integer");
      }
      validateChecks(phase.preconditions);
      if (phase.postconditions.length === 0) {
        invalidPhase("data_backfill requires at least one postcondition");
      }
      validateChecks(phase.postconditions);
      return;
    case "validation":
      if (phase.checks.length === 0) {
        invalidPhase("validation requires at least one check");
      }
      validateChecks(phase.checks);
      return;
    case "traffic_transition":
      requireText(phase.deployment_gate, "deployment_gate");
      return;
    case "destructive_cleanup":
      if (phase.metadata.approval !== "destructive_operator") {
        invalidPhase(
          "destructive_cleanup requires destructive_operator approval",
        );
      }
      requireText(phase.backup_requirement, "backup_requirement");
      validateStatements(phase.statements);
      validateChecks(phase.preconditions);
      validateChecks(phase.postconditions);
      return;
  }
}

function validateStatements(statements: MigrationStatement[]): void {
  if (statements.length === 0) {
    invalidPhase("DDL phase requires at least one statement");
  }
  statements.forEach((statement, index) => {
    if (statement.ordinal !== index + 1) {
      invalidPhase("statement ordinals must be contiguous and start at one");
    }
    requireText(statement.sql, "statement.sql");
    requireSha256(statement.sha256, "statement.sha256");
  });
}

function validateChecks(checks: MigrationCheck[]): void {
  for (const check of checks) {
    requireText(check.name, "check.name");
    requireText(check.sql, "check.sql");
    if (check.expectation === "scalar_equals" && check.expected_value === undefined) {
      invalidPhase("scalar_equals checks require expected_value");
    }
    if (check.expectation !== "scalar_equals" && check.expected_value !== undefined) {
      invalidPhase("expected_value is allowed only for scalar_equals checks");
    }
  }
}

function requireText(value: string, field: string): void {
  if (!value.trim()) {
    throw new InterfaceError("empty_field", `${field} must be non-empty`);
  }
}

function requireSha256(value: string, field: string): void {
  if (!SHA256.test(value)) {
    throw new InterfaceError(
      "invalid_sha256",
      `${field} must be a lowercase hexadecimal SHA-256 digest`,
    );
  }
}

function invalidPhase(message: string): never {
  throw new InterfaceError("invalid_phase", message);
}
