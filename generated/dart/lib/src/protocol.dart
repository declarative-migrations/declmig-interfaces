import 'errors.dart';
import 'models.dart';

final RegExp _sha256 = RegExp(r'^[a-f0-9]{64}$');

MigrationPlan validateMigrationPlan(MigrationPlan plan) {
  if (plan.format != planFormat) {
    throw const InterfaceException(
      'invalid_format',
      'unsupported migration-plan format',
    );
  }
  _requireText(plan.id, 'id');
  _requireText(plan.revision, 'revision');
  _requireText(plan.engineVersion, 'engine_version');
  _requireSha256(plan.sourceCatalogSha256, 'source_catalog_sha256');
  _requireSha256(plan.desiredCatalogSha256, 'desired_catalog_sha256');
  _requireSha256(plan.renderedSqlSha256, 'rendered_sql_sha256');
  if (plan.phases.isEmpty) {
    throw const InterfaceException(
      'empty_phases',
      'at least one phase is required',
    );
  }

  final phaseIds = <String>{};
  for (final phase in plan.phases) {
    final destructiveCleanup = phase is DestructiveCleanupPhase;
    _validateMetadata(phase.metadata, destructiveCleanup);
    if (!phaseIds.add(phase.metadata.id)) {
      throw const InterfaceException(
        'duplicate_phase_id',
        'phase ids must be unique',
      );
    }
    _validatePhase(phase, plan.engine);
  }
  return plan;
}

void _validateMetadata(
  PhaseMetadata metadata,
  bool destructiveCleanup,
) {
  _requireText(metadata.id, 'phase.metadata.id');
  if (metadata.timeoutSeconds < 1) {
    _invalidPhase('timeout_seconds must be a positive integer');
  }
  if (metadata.changes.isEmpty) {
    _invalidPhase('phase metadata requires at least one change');
  }

  for (final change in metadata.changes) {
    _requireText(change.resource, 'change.resource');
    if (change.safety == SafetyClass.manualReview &&
        (change.manualReason == null || change.manualReason!.trim().isEmpty)) {
      _invalidPhase('manual_review changes require manual_reason');
    }
    if (change.safety == SafetyClass.destructive && !destructiveCleanup) {
      _invalidPhase('destructive changes must use destructive_cleanup');
    }
  }
}

void _validatePhase(MigrationPhase phase, DatabaseEngine engine) {
  if (phase is TransactionalDdlPhase) {
    _validateStatements(phase.statements);
    _validateChecks(phase.preconditions);
    _validateChecks(phase.postconditions);
    return;
  }
  if (phase is NonTransactionalDdlPhase) {
    _validateStatements(phase.statements);
    _requireText(phase.cleanupOnFailure, 'cleanup_on_failure');
    _validateChecks(phase.preconditions);
    _validateChecks(phase.postconditions);
    return;
  }
  if (phase is CockroachSchemaJobPhase) {
    if (engine != DatabaseEngine.cockroachdb) {
      _invalidPhase('cockroach_schema_job requires engine=cockroachdb');
    }
    _validateStatements(phase.statements);
    _validateChecks(phase.preconditions);
    _validateChecks(phase.postconditions);
    return;
  }
  if (phase is DataBackfillPhase) {
    _requireSha256(
      phase.workerArtifactSha256,
      'worker_artifact_sha256',
    );
    _requireText(phase.batchKey, 'batch_key');
    if (phase.maxBatchSize < 1) {
      _invalidPhase('max_batch_size must be a positive integer');
    }
    _validateChecks(phase.preconditions);
    if (phase.postconditions.isEmpty) {
      _invalidPhase('data_backfill requires at least one postcondition');
    }
    _validateChecks(phase.postconditions);
    return;
  }
  if (phase is ValidationPhase) {
    if (phase.checks.isEmpty) {
      _invalidPhase('validation requires at least one check');
    }
    _validateChecks(phase.checks);
    return;
  }
  if (phase is TrafficTransitionPhase) {
    _requireText(phase.deploymentGate, 'deployment_gate');
    return;
  }
  if (phase is DestructiveCleanupPhase) {
    if (phase.metadata.approval !=
        ApprovalRequirement.destructiveOperator) {
      _invalidPhase(
        'destructive_cleanup requires destructive_operator approval',
      );
    }
    _requireText(phase.backupRequirement, 'backup_requirement');
    _validateStatements(phase.statements);
    _validateChecks(phase.preconditions);
    _validateChecks(phase.postconditions);
    return;
  }
  _invalidPhase('unsupported migration phase');
}

void _validateStatements(List<MigrationStatement> statements) {
  if (statements.isEmpty) {
    _invalidPhase('DDL phase requires at least one statement');
  }
  for (var index = 0; index < statements.length; index += 1) {
    final statement = statements[index];
    if (statement.ordinal != index + 1) {
      _invalidPhase(
        'statement ordinals must be contiguous and start at one',
      );
    }
    _requireText(statement.sql, 'statement.sql');
    _requireSha256(statement.sha256, 'statement.sha256');
  }
}

void _validateChecks(List<MigrationCheck> checks) {
  for (final check in checks) {
    _requireText(check.name, 'check.name');
    _requireText(check.sql, 'check.sql');
    if (check.expectation == CheckExpectation.scalarEquals &&
        check.expectedValue == null) {
      _invalidPhase('scalar_equals checks require expected_value');
    }
    if (check.expectation != CheckExpectation.scalarEquals &&
        check.expectedValue != null) {
      _invalidPhase(
        'expected_value is allowed only for scalar_equals checks',
      );
    }
  }
}

void _requireText(String value, String field) {
  if (value.trim().isEmpty) {
    throw InterfaceException('empty_field', '$field must be non-empty');
  }
}

void _requireSha256(String value, String field) {
  if (!_sha256.hasMatch(value)) {
    throw InterfaceException(
      'invalid_sha256',
      '$field must be a lowercase hexadecimal SHA-256 digest',
    );
  }
}

Never _invalidPhase(String message) {
  throw InterfaceException('invalid_phase', message);
}
