//! Type definitions for OverDrive-DB Rust SDK

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Query result containing rows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Result rows as JSON values
    pub rows: Vec<Value>,
}

impl QueryResult {
    /// Get number of rows
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    
    /// Check if result is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    
    /// Get first row if available
    pub fn first(&self) -> Option<&Value> {
        self.rows.first()
    }
    
    /// Iterate over rows
    pub fn iter(&self) -> std::slice::Iter<Value> {
        self.rows.iter()
    }
}

impl IntoIterator for QueryResult {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

/// Storage engine types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageEngine {
    /// Persistent disk storage (default)
    Disk,
    /// In-memory storage with snapshots
    RAM,
    /// Vector similarity search
    Vector,
    /// Time-series data with compression
    TimeSeries,
    /// Graph database with traversal
    Graph,
    /// Event streaming with topics
    Streaming,
}

impl StorageEngine {
    /// Get engine name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageEngine::Disk => "DISK",
            StorageEngine::RAM => "RAM",
            StorageEngine::Vector => "VECTOR",
            StorageEngine::TimeSeries => "TIMESERIES",
            StorageEngine::Graph => "GRAPH",
            StorageEngine::Streaming => "STREAMING",
        }
    }
}

impl std::fmt::Display for StorageEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Read uncommitted data (fastest, least safe)
    ReadUncommitted,
    /// Read committed data only (default)
    ReadCommitted,
    /// Repeatable reads with snapshot isolation
    RepeatableRead,
    /// Full serializable isolation (slowest, safest)
    Serializable,
}

impl IsolationLevel {
    /// Get isolation level name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ_UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ_COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE_READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

impl Default for IsolationLevel {
    fn default() -> Self {
        IsolationLevel::ReadCommitted
    }
}

/// Transaction options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOptions {
    /// Isolation level for the transaction
    pub isolation_level: IsolationLevel,
    /// Timeout in milliseconds (None for no timeout)
    pub timeout_ms: Option<u64>,
    /// Whether to enable deadlock detection
    pub deadlock_detection: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation_level: IsolationLevel::default(),
            timeout_ms: None,
            deadlock_detection: true,
        }
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    /// Number of tables
    pub table_count: usize,
    /// Total number of documents
    pub document_count: usize,
    /// Database file size in bytes
    pub file_size_bytes: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    /// Total queries executed
    pub total_queries: u64,
    /// Average query time in milliseconds
    pub avg_query_time_ms: f64,
}

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    /// Index name
    pub name: String,
    /// Table name
    pub table: String,
    /// Indexed columns/fields
    pub columns: Vec<String>,
    /// Whether index is unique
    pub unique: bool,
    /// Index type (btree, hash, text, etc.)
    pub index_type: IndexType,
}

/// Index types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    /// B-tree index for range queries
    BTree,
    /// Hash index for exact matches
    Hash,
    /// Full-text search index
    Text,
    /// Vector similarity index
    Vector,
}

impl IndexType {
    /// Get index type name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            IndexType::BTree => "BTREE",
            IndexType::Hash => "HASH",
            IndexType::Text => "TEXT",
            IndexType::Vector => "VECTOR",
        }
    }
}

/// Search options for full-text search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Fields to search in (None for all fields)
    pub fields: Option<Vec<String>>,
    /// Enable fuzzy matching
    pub fuzzy: bool,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Enable result highlighting
    pub highlight: bool,
    /// Minimum similarity score (0.0 to 1.0)
    pub min_score: Option<f64>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            fields: None,
            fuzzy: false,
            limit: None,
            highlight: false,
            min_score: None,
        }
    }
}

/// Vector search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchOptions {
    /// Number of results to return
    pub limit: usize,
    /// Similarity threshold (0.0 to 1.0)
    pub threshold: Option<f64>,
    /// Distance metric
    pub metric: VectorMetric,
}

/// Vector distance metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorMetric {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Manhattan distance
    Manhattan,
    /// Dot product
    DotProduct,
}

impl VectorMetric {
    /// Get metric name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            VectorMetric::Cosine => "cosine",
            VectorMetric::Euclidean => "euclidean",
            VectorMetric::Manhattan => "manhattan",
            VectorMetric::DotProduct => "dot_product",
        }
    }
}

impl Default for VectorSearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            threshold: None,
            metric: VectorMetric::Cosine,
        }
    }
}