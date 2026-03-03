use modkit_db::DbError;
use modkit_db::secure::ScopeError;
use modkit_macros::domain_model;
use thiserror::Error;
use uuid::Uuid;

#[domain_model]
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Conflict: {code}: {message}")]
    Conflict { code: String, message: String },

    #[error("{entity} not found: {id}")]
    NotFound { entity: String, id: Uuid },

    #[error("Access denied")]
    Forbidden,

    #[error("Internal error")]
    Internal,
}

impl DomainError {
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Conflict {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn not_found(entity: impl Into<String>, id: Uuid) -> Self {
        Self::NotFound {
            entity: entity.into(),
            id,
        }
    }
}

/// Helper to convert any displayable error into `DomainError::Database`.
pub fn db_err(e: impl std::fmt::Display) -> DomainError {
    DomainError::database(e.to_string())
}

impl From<DbError> for DomainError {
    fn from(e: DbError) -> Self {
        DomainError::database(e.to_string())
    }
}

impl From<ScopeError> for DomainError {
    #[allow(clippy::cognitive_complexity)]
    fn from(e: ScopeError) -> Self {
        match e {
            ScopeError::Db(ref db_err) => map_db_err(db_err),
            ScopeError::Denied(msg) => {
                tracing::warn!("scope denied: {msg}");
                DomainError::Forbidden
            }
            ScopeError::TenantNotInScope { tenant_id } => {
                tracing::warn!("tenant {tenant_id} not in scope");
                DomainError::Forbidden
            }
            ScopeError::Invalid(msg) => {
                tracing::error!("invalid scope: {msg}");
                DomainError::Internal
            }
        }
    }
}

fn map_db_err(db_err: &sea_orm::DbErr) -> DomainError {
    if let Some(sea_orm::SqlErr::UniqueConstraintViolation(msg)) = db_err.sql_err() {
        return DomainError::Conflict {
            code: "unique_violation".into(),
            message: msg,
        };
    }
    DomainError::database(db_err.to_string())
}
