const protocolVersion = '2';
const schemaRevision = 'declmig-0002';
const planFormat = 'ores.migration-plan/v2';

enum DatabaseEngine { postgresql, cockroachdb }

enum SafetyClass { safe, online, blocking, destructive, manualReview }

enum RetryClass { never, idempotent, serializableRestart, replanRequired }

enum RollbackClass { reversible, compensatable, restoreRequired, irreversible }

enum ApprovalRequirement {
  none,
  operator,
  destructiveOperator,
  securityReview,
  manual,
}

enum LockExpectation { none, metadata, share, exclusive, engineManaged }

enum ChangeKind {
  createSchema,
  dropSchema,
  createTable,
  dropTable,
  addColumn,
  alterColumn,
  dropColumn,
  addConstraint,
  dropConstraint,
  createIndex,
  dropIndex,
  createView,
  dropView,
  createFunction,
  dropFunction,
  dataBackfill,
  manual,
}

enum CheckExpectation { noRows, booleanTrue, scalarEquals }

class Health {
  const Health({
    required this.ok,
    required this.service,
    required this.protocol,
  });

  final bool ok;
  final String service;
  final String protocol;
}

class MigrationChange {
  const MigrationChange({
    required this.kind,
    required this.resource,
    required this.safety,
    this.manualReason,
  });

  final ChangeKind kind;
  final String resource;
  final SafetyClass safety;
  final String? manualReason;
}

class PhaseMetadata {
  const PhaseMetadata({
    required this.id,
    required this.safety,
    required this.retry,
    required this.rollback,
    required this.approval,
    required this.timeoutSeconds,
    required this.lockExpectation,
    required this.changes,
  });

  final String id;
  final SafetyClass safety;
  final RetryClass retry;
  final RollbackClass rollback;
  final ApprovalRequirement approval;
  final int timeoutSeconds;
  final LockExpectation lockExpectation;
  final List<MigrationChange> changes;
}

class MigrationStatement {
  const MigrationStatement({
    required this.ordinal,
    required this.sql,
    required this.sha256,
  });

  final int ordinal;
  final String sql;
  final String sha256;
}

class MigrationCheck {
  const MigrationCheck({
    required this.name,
    required this.sql,
    required this.expectation,
    this.expectedValue,
  });

  final String name;
  final String sql;
  final CheckExpectation expectation;
  final String? expectedValue;
}

sealed class MigrationPhase {
  const MigrationPhase({required this.metadata});

  final PhaseMetadata metadata;
}

class TransactionalDdlPhase extends MigrationPhase {
  const TransactionalDdlPhase({
    required super.metadata,
    required this.statements,
    required this.preconditions,
    required this.postconditions,
  });

  final List<MigrationStatement> statements;
  final List<MigrationCheck> preconditions;
  final List<MigrationCheck> postconditions;
}

class NonTransactionalDdlPhase extends MigrationPhase {
  const NonTransactionalDdlPhase({
    required super.metadata,
    required this.statements,
    required this.cleanupOnFailure,
    required this.preconditions,
    required this.postconditions,
  });

  final List<MigrationStatement> statements;
  final String cleanupOnFailure;
  final List<MigrationCheck> preconditions;
  final List<MigrationCheck> postconditions;
}

class CockroachSchemaJobPhase extends MigrationPhase {
  const CockroachSchemaJobPhase({
    required super.metadata,
    required this.statements,
    required this.waitForTerminalJobs,
    required this.preconditions,
    required this.postconditions,
  });

  final List<MigrationStatement> statements;
  final bool waitForTerminalJobs;
  final List<MigrationCheck> preconditions;
  final List<MigrationCheck> postconditions;
}

class DataBackfillPhase extends MigrationPhase {
  const DataBackfillPhase({
    required super.metadata,
    required this.workerArtifactSha256,
    required this.batchKey,
    required this.maxBatchSize,
    required this.preconditions,
    required this.postconditions,
  });

  final String workerArtifactSha256;
  final String batchKey;
  final int maxBatchSize;
  final List<MigrationCheck> preconditions;
  final List<MigrationCheck> postconditions;
}

class ValidationPhase extends MigrationPhase {
  const ValidationPhase({
    required super.metadata,
    required this.checks,
  });

  final List<MigrationCheck> checks;
}

class TrafficTransitionPhase extends MigrationPhase {
  const TrafficTransitionPhase({
    required super.metadata,
    required this.deploymentGate,
  });

  final String deploymentGate;
}

class DestructiveCleanupPhase extends MigrationPhase {
  const DestructiveCleanupPhase({
    required super.metadata,
    required this.statements,
    required this.backupRequirement,
    required this.preconditions,
    required this.postconditions,
  });

  final List<MigrationStatement> statements;
  final String backupRequirement;
  final List<MigrationCheck> preconditions;
  final List<MigrationCheck> postconditions;
}

class MigrationPlan {
  const MigrationPlan({
    required this.format,
    required this.id,
    required this.revision,
    required this.engine,
    required this.engineVersion,
    required this.sourceCatalogSha256,
    required this.desiredCatalogSha256,
    required this.renderedSqlSha256,
    required this.phases,
  });

  final String format;
  final String id;
  final String revision;
  final DatabaseEngine engine;
  final String engineVersion;
  final String sourceCatalogSha256;
  final String desiredCatalogSha256;
  final String renderedSqlSha256;
  final List<MigrationPhase> phases;
}
