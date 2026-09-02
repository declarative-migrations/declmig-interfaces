#![forbid(unsafe_code)]

//! Data-only contracts. Implementations live in clients, servers, and lib-core.

pub mod error;
pub mod protocol;
pub mod schema;

pub use error::InterfaceError;
pub use protocol::{
    ApprovalRequirement, ChangeKind, CheckExpectation, CockroachSchemaJobPhase,
    DataBackfillPhase, DatabaseEngine, DestructiveCleanupPhase, Health, LockExpectation,
    MigrationChange, MigrationCheck, MigrationPhase, MigrationPlan, MigrationStatement,
    NonTransactionalDdlPhase, PhaseMetadata, RetryClass, RollbackClass, SafetyClass,
    TrafficTransitionPhase, TransactionalDdlPhase, ValidationPhase, PLAN_FORMAT,
    PROTOCOL_VERSION,
};
pub use schema::{SCHEMA_ID, SCHEMA_REVISION};
