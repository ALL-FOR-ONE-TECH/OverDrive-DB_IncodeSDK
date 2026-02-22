//! SDK-specific error types for OverDrive InCode SDK
//! 
//! Provides clean, user-friendly error types that wrap the internal engine errors.

use std::fmt;

/// SDK Result type
pub type SdkResult<T> = std::result::Result<T, SdkError>;

/// SDK Error types — clean, user-friendly errors
#[derive(Debug)]
pub enum SdkError {
    /// Database file not found
    DatabaseNotFound(String),
    /// Database already exists
    DatabaseAlreadyExists(String),
    /// Table not found
    TableNotFound(String),
    /// Table already exists
    TableAlreadyExists(String),
    /// Document/record not found
    DocumentNotFound(String),
    /// Invalid SQL query
    InvalidQuery(String),
    /// I/O error (file operations)
    IoError(std::io::Error),
    /// Serialization/deserialization error
    SerializationError(String),
    /// Constraint violation (unique, foreign key, etc.)
    ConstraintViolation(String),
    /// Transaction error
    TransactionError(String),
    /// Database is closed
    DatabaseClosed,
    /// Internal engine error
    Internal(String),
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::DatabaseNotFound(p) => write!(f, "Database not found: {}", p),
            SdkError::DatabaseAlreadyExists(p) => write!(f, "Database already exists: {}", p),
            SdkError::TableNotFound(t) => write!(f, "Table not found: {}", t),
            SdkError::TableAlreadyExists(t) => write!(f, "Table already exists: {}", t),
            SdkError::DocumentNotFound(id) => write!(f, "Document not found: {}", id),
            SdkError::InvalidQuery(q) => write!(f, "Invalid query: {}", q),
            SdkError::IoError(e) => write!(f, "I/O error: {}", e),
            SdkError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            SdkError::ConstraintViolation(c) => write!(f, "Constraint violation: {}", c),
            SdkError::TransactionError(e) => write!(f, "Transaction error: {}", e),
            SdkError::DatabaseClosed => write!(f, "Database is closed"),
            SdkError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for SdkError {}

impl From<std::io::Error> for SdkError {
    fn from(e: std::io::Error) -> Self {
        SdkError::IoError(e)
    }
}

impl From<overdrive_db::error::DatabaseError> for SdkError {
    fn from(e: overdrive_db::error::DatabaseError) -> Self {
        match e {
            overdrive_db::error::DatabaseError::Io(io_err) => SdkError::IoError(io_err),
            overdrive_db::error::DatabaseError::Corrupted(msg) => SdkError::Internal(msg),
            overdrive_db::error::DatabaseError::KeyNotFound(msg) => SdkError::DocumentNotFound(msg),
            overdrive_db::error::DatabaseError::TableNotFound(msg) => SdkError::TableNotFound(msg),
            overdrive_db::error::DatabaseError::Connection(msg) => SdkError::Internal(msg),
            overdrive_db::error::DatabaseError::Query(msg) => SdkError::InvalidQuery(msg),
            overdrive_db::error::DatabaseError::TransactionError(msg) => SdkError::TransactionError(msg),
            overdrive_db::error::DatabaseError::Validation(msg) => SdkError::ConstraintViolation(msg),
            overdrive_db::error::DatabaseError::Json(e) => SdkError::SerializationError(e.to_string()),
            overdrive_db::error::DatabaseError::Serialization(e) => SdkError::SerializationError(e.to_string()),
            _ => SdkError::Internal(format!("{}", e)),
        }
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(e: serde_json::Error) -> Self {
        SdkError::SerializationError(e.to_string())
    }
}
