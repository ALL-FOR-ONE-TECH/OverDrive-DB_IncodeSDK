//! Error types for OverDrive-DB Rust SDK

use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, OverDriveError>;

/// OverDrive-DB error types
#[derive(Error, Debug)]
pub enum OverDriveError {
    /// Database operation error
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Query execution error
    #[error("Query error: {0}")]
    QueryError(String),
    
    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    /// Authentication/authorization error
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// Table operation error
    #[error("Table error: {0}")]
    TableError(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),
    
    /// Native library loading error
    #[error("Library error: {0}")]
    LibraryError(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// String conversion error
    #[error("String conversion error: {0}")]
    StringError(#[from] std::ffi::NulError),
    
    /// Invalid parameter error
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Timeout error
    #[error("Operation timed out: {0}")]
    TimeoutError(String),
    
    /// Constraint violation error
    #[error("Constraint violation: {0}")]
    ConstraintError(String),
    
    /// Index error
    #[error("Index error: {0}")]
    IndexError(String),
    
    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    /// Version mismatch error
    #[error("Version mismatch: {0}")]
    VersionError(String),
    
    /// Generic error
    #[error("Error: {0}")]
    Other(String),
}

impl OverDriveError {
    /// Create a database error
    pub fn database<S: Into<String>>(msg: S) -> Self {
        OverDriveError::DatabaseError(msg.into())
    }
    
    /// Create a query error
    pub fn query<S: Into<String>>(msg: S) -> Self {
        OverDriveError::QueryError(msg.into())
    }
    
    /// Create a transaction error
    pub fn transaction<S: Into<String>>(msg: S) -> Self {
        OverDriveError::TransactionError(msg.into())
    }
    
    /// Create an authentication error
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        OverDriveError::AuthError(msg.into())
    }
    
    /// Create a table error
    pub fn table<S: Into<String>>(msg: S) -> Self {
        OverDriveError::TableError(msg.into())
    }
    
    /// Create an I/O error
    pub fn io<S: Into<String>>(msg: S) -> Self {
        OverDriveError::IoError(msg.into())
    }
    
    /// Create a library error
    pub fn library<S: Into<String>>(msg: S) -> Self {
        OverDriveError::LibraryError(msg.into())
    }
    
    /// Create an invalid parameter error
    pub fn invalid_parameter<S: Into<String>>(msg: S) -> Self {
        OverDriveError::InvalidParameter(msg.into())
    }
    
    /// Create a connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        OverDriveError::ConnectionError(msg.into())
    }
    
    /// Create a timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        OverDriveError::TimeoutError(msg.into())
    }
    
    /// Create a constraint error
    pub fn constraint<S: Into<String>>(msg: S) -> Self {
        OverDriveError::ConstraintError(msg.into())
    }
    
    /// Create an index error
    pub fn index<S: Into<String>>(msg: S) -> Self {
        OverDriveError::IndexError(msg.into())
    }
    
    /// Create an encryption error
    pub fn encryption<S: Into<String>>(msg: S) -> Self {
        OverDriveError::EncryptionError(msg.into())
    }
    
    /// Create a version error
    pub fn version<S: Into<String>>(msg: S) -> Self {
        OverDriveError::VersionError(msg.into())
    }
    
    /// Create a generic error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        OverDriveError::Other(msg.into())
    }
    
    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            OverDriveError::DatabaseError(_) => ErrorCategory::Database,
            OverDriveError::QueryError(_) => ErrorCategory::Query,
            OverDriveError::TransactionError(_) => ErrorCategory::Transaction,
            OverDriveError::AuthError(_) => ErrorCategory::Authentication,
            OverDriveError::TableError(_) => ErrorCategory::Table,
            OverDriveError::IoError(_) => ErrorCategory::Io,
            OverDriveError::LibraryError(_) => ErrorCategory::Library,
            OverDriveError::JsonError(_) => ErrorCategory::Serialization,
            OverDriveError::StringError(_) => ErrorCategory::Serialization,
            OverDriveError::InvalidParameter(_) => ErrorCategory::Parameter,
            OverDriveError::ConnectionError(_) => ErrorCategory::Connection,
            OverDriveError::TimeoutError(_) => ErrorCategory::Timeout,
            OverDriveError::ConstraintError(_) => ErrorCategory::Constraint,
            OverDriveError::IndexError(_) => ErrorCategory::Index,
            OverDriveError::EncryptionError(_) => ErrorCategory::Encryption,
            OverDriveError::VersionError(_) => ErrorCategory::Version,
            OverDriveError::Other(_) => ErrorCategory::Other,
        }
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            OverDriveError::TimeoutError(_) => true,
            OverDriveError::ConnectionError(_) => true,
            OverDriveError::TransactionError(_) => true,
            OverDriveError::ConstraintError(_) => true,
            _ => false,
        }
    }
    
    /// Check if error is temporary
    pub fn is_temporary(&self) -> bool {
        match self {
            OverDriveError::TimeoutError(_) => true,
            OverDriveError::ConnectionError(_) => true,
            _ => false,
        }
    }
}

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Database-level errors
    Database,
    /// Query execution errors
    Query,
    /// Transaction errors
    Transaction,
    /// Authentication/authorization errors
    Authentication,
    /// Table operation errors
    Table,
    /// I/O errors
    Io,
    /// Native library errors
    Library,
    /// Serialization errors
    Serialization,
    /// Parameter validation errors
    Parameter,
    /// Connection errors
    Connection,
    /// Timeout errors
    Timeout,
    /// Constraint violation errors
    Constraint,
    /// Index errors
    Index,
    /// Encryption errors
    Encryption,
    /// Version compatibility errors
    Version,
    /// Other errors
    Other,
}

impl ErrorCategory {
    /// Get category name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::Database => "database",
            ErrorCategory::Query => "query",
            ErrorCategory::Transaction => "transaction",
            ErrorCategory::Authentication => "authentication",
            ErrorCategory::Table => "table",
            ErrorCategory::Io => "io",
            ErrorCategory::Library => "library",
            ErrorCategory::Serialization => "serialization",
            ErrorCategory::Parameter => "parameter",
            ErrorCategory::Connection => "connection",
            ErrorCategory::Timeout => "timeout",
            ErrorCategory::Constraint => "constraint",
            ErrorCategory::Index => "index",
            ErrorCategory::Encryption => "encryption",
            ErrorCategory::Version => "version",
            ErrorCategory::Other => "other",
        }
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for OverDriveError {
    fn from(err: std::io::Error) -> Self {
        OverDriveError::IoError(err.to_string())
    }
}

/// Convert from libloading::Error
impl From<libloading::Error> for OverDriveError {
    fn from(err: libloading::Error) -> Self {
        OverDriveError::LibraryError(err.to_string())
    }
}