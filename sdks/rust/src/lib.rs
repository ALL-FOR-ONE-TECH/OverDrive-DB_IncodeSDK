//! # OverDrive-DB Rust SDK v2.0.0
//! 
//! Embeddable hybrid SQL+NoSQL database with 6 storage engines.
//! 
//! ## Features
//! 
//! - **Hybrid Schema**: Both schema-based and schemaless modes
//! - **MVCC Transactions**: Multi-version concurrency control (automatic)
//! - **ACID Compliance**: Atomicity, Consistency, Isolation, Durability (automatic)
//! - **6 Storage Engines**: Disk, RAM, Vector, Time-Series, Graph, Streaming
//! - **Full-Text Search**: Built-in text search with fuzzy matching
//! - **Encryption**: AES-256-GCM with Argon2id key derivation
//! - **Cross-Platform**: Windows, Linux, macOS (x64 and ARM64)
//! 
//! ## Quick Start
//! 
//! ```rust
//! use overdrive::OverDrive;
//! use serde_json::json;
//! 
//! // Open database (all features automatic)
//! let mut odb = OverDrive::open("myapp.odb")?;
//! 
//! // Create table (schemaless by default)
//! odb.create_table("users")?;
//! 
//! // Insert document
//! let id = odb.insert("users", &json!({
//!     "name": "Alice",
//!     "age": 30,
//!     "email": "alice@example.com"
//! }))?;
//! 
//! // Query data
//! let users = odb.query("SELECT * FROM users WHERE age > 25")?;
//! println!("Users: {}", users);
//! 
//! // Close database
//! odb.close()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod client;
pub mod loader;
pub mod types;
pub mod errors;

// Re-export main types
pub use client::OverDrive;
pub use types::{QueryResult, TransactionOptions, StorageEngine};
pub use errors::{OverDriveError, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get library version
pub fn version() -> &'static str {
    VERSION
}