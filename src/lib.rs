#![forbid(unsafe_code)]

//! Data-only contracts. Implementations live in clients, servers, and lib-core.

pub mod error;
pub mod migration_plan_v2;
pub mod parity;
pub mod protocol;
pub mod schema;

pub use error::InterfaceError;
pub use migration_plan_v2::{
    MigrationPlanV2, MigrationPlanV2Error, MIGRATION_PLAN_FORMAT, MIGRATION_PLAN_PROTOCOL_VERSION,
};
pub use parity::{
    ComparisonEvidence, ComparisonKind, ComparisonStatus, ParityDecision,
    PeerAuthorityCertification, PeerAuthorityInputs, PeerAuthorityPolicy,
    PEER_AUTHORITY_CERTIFICATION_FORMAT,
};
pub use protocol::{Health, MigrationPlan, PROTOCOL_VERSION};
pub use schema::{SCHEMA_ID, SCHEMA_REVISION};
