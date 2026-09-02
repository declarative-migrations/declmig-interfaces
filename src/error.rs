#![forbid(unsafe_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceError {
    EmptyId,
    EmptyRevision,
    InvalidFormat,
    EmptyField(&'static str),
    InvalidSha256(&'static str),
    EmptyPhases,
    DuplicatePhaseId,
    InvalidPhase(&'static str),
    SchemaMismatch,
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(f, "id must be non-empty"),
            Self::EmptyRevision => write!(f, "revision must be non-empty"),
            Self::InvalidFormat => write!(f, "migration plan format is unsupported"),
            Self::EmptyField(field) => write!(f, "{field} must be non-empty"),
            Self::InvalidSha256(field) => {
                write!(f, "{field} must be a lowercase hexadecimal SHA-256 digest")
            }
            Self::EmptyPhases => write!(f, "migration plan must contain at least one phase"),
            Self::DuplicatePhaseId => write!(f, "migration phase ids must be unique"),
            Self::InvalidPhase(reason) => write!(f, "migration phase is invalid: {reason}"),
            Self::SchemaMismatch => write!(f, "payload does not match the published schema"),
        }
    }
}

impl std::error::Error for InterfaceError {}
