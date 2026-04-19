/**
 * OverDrive InCode SDK — C/C++ Header (v1.4.3)
 *
 * Embeddable hybrid SQL+NoSQL document database. Like SQLite for JSON.
 *
 * Link against:
 *   Windows : overdrive.dll
 *   Linux   : liboverdrive.so
 *   macOS   : liboverdrive.dylib
 *
 * Download from:
 *   https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest
 *
 * CMake integration:
 *   find_library(OVERDRIVE_LIB overdrive HINTS ${CMAKE_SOURCE_DIR}/lib)
 *   target_link_libraries(your_target ${OVERDRIVE_LIB})
 *   target_include_directories(your_target PRIVATE ${CMAKE_SOURCE_DIR}/include)
 *
 * Memory rules:
 *   - Every char* returned by overdrive_* MUST be freed with overdrive_free_string()
 *   - ODB* handles MUST be closed with overdrive_close()
 *   - overdrive_last_error() and overdrive_version() return static strings — do NOT free
 *
 * Quick start:
 *   ODB* db = overdrive_open("myapp.odb");
 *   overdrive_create_table(db, "users");
 *   char* id = overdrive_insert(db, "users", "{\"name\":\"Alice\",\"age\":30}");
 *   overdrive_free_string(id);
 *   char* result = overdrive_query(db, "SELECT * FROM users WHERE age > 25");
 *   printf("%s\n", result);
 *   overdrive_free_string(result);
 *   overdrive_close(db);
 */

#ifndef OVERDRIVE_H
#define OVERDRIVE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Opaque database handle. */
typedef void* ODB;

/* ═══════════════════════════════════════════════════════════
 * LIFECYCLE
 * ═══════════════════════════════════════════════════════════ */

/**
 * Open (or create) a database at path.
 * File permissions are automatically hardened (chmod 600 / Windows ACL).
 * Returns NULL on error — check overdrive_last_error().
 */
ODB overdrive_open(const char* path);

/**
 * Open (or create) a database with engine selection and options. (v1.4)
 *
 * engine: "Disk" | "RAM" | "Vector" | "Time-Series" | "Graph" | "Streaming"
 * options_json: JSON object, e.g. {"password":"secret","auto_create_tables":true}
 *   - password (string, optional): AES-256-GCM encryption via Argon2id key derivation
 *   - auto_create_tables (bool, default true): create tables on first insert
 *
 * Returns NULL on error — check overdrive_last_error().
 */
ODB overdrive_open_with_engine(const char* path, const char* engine, const char* options_json);

/**
 * Open (or create) a password-protected database (v1.4).
 *
 * Derives the AES-256-GCM encryption key using Argon2id.
 * password must be at least 8 characters. Pass NULL for no encryption.
 *
 * Returns NULL on error — check overdrive_last_error().
 */
ODB overdrive_open_with_password(const char* path, const char* password);

/**
 * Close the database and release all resources.
 */
void overdrive_close(ODB db);

/**
 * Force sync all data to disk.
 */
void overdrive_sync(ODB db);

/**
 * Get the SDK version string. Do NOT free this pointer.
 */
const char* overdrive_version(void);

/* ═══════════════════════════════════════════════════════════
 * TABLE MANAGEMENT
 * ═══════════════════════════════════════════════════════════ */

/**
 * Create a new table (Disk engine). Returns 0 on success, -1 on error.
 */
int overdrive_create_table(ODB db, const char* name);

/**
 * Create a table with a specific storage engine. (v1.4)
 * engine: "Disk" | "RAM" | "Vector" | "Time-Series" | "Graph" | "Streaming"
 * Returns 0 on success, -1 on error.
 */
int overdrive_create_table_with_engine(ODB db, const char* name, const char* engine);

/**
 * Drop a table and all its data. Returns 0 on success, -1 on error.
 */
int overdrive_drop_table(ODB db, const char* name);

/**
 * List all tables as a JSON array string. Must be freed with overdrive_free_string().
 * Returns NULL on error.
 */
char* overdrive_list_tables(ODB db);

/**
 * Check if a table exists. Returns 1 if exists, 0 if not, -1 on error.
 */
int overdrive_table_exists(ODB db, const char* name);

/* ═══════════════════════════════════════════════════════════
 * CRUD OPERATIONS
 * ═══════════════════════════════════════════════════════════ */

/**
 * Insert a JSON document into a table.
 * Returns the generated _id string (e.g. "users_1"). Must be freed.
 * Returns NULL on error.
 */
char* overdrive_insert(ODB db, const char* table, const char* json_doc);

/**
 * Get a document by _id.
 * Returns JSON string. Must be freed. Returns NULL if not found.
 */
char* overdrive_get(ODB db, const char* table, const char* id);

/**
 * Update document fields by _id. json_updates is a JSON object with fields to merge.
 * Returns 1 if updated, 0 if not found, -1 on error.
 */
int overdrive_update(ODB db, const char* table, const char* id, const char* json_updates);

/**
 * Delete a document by _id.
 * Returns 1 if deleted, 0 if not found, -1 on error.
 */
int overdrive_delete(ODB db, const char* table, const char* id);

/**
 * Count documents in a table.
 * Returns count >= 0 on success, -1 on error.
 */
int overdrive_count(ODB db, const char* table);

/* ═══════════════════════════════════════════════════════════
 * QUERY ENGINE
 * ═══════════════════════════════════════════════════════════ */

/**
 * Execute an SQL query.
 * Returns JSON: {"rows":[...],"columns":[...],"rows_affected":N,"execution_time_ms":N}
 * Must be freed. Returns NULL on error.
 *
 * Supported SQL:
 *   SELECT * FROM table [WHERE ...] [ORDER BY ...] [LIMIT N] [OFFSET N]
 *   SELECT COUNT(*), AVG(col), SUM(col), MIN(col), MAX(col) FROM table
 *   INSERT INTO table VALUES {json}
 *   UPDATE table SET {json} WHERE ...
 *   DELETE FROM table WHERE ...
 *   CREATE TABLE name
 *   DROP TABLE name
 *   SHOW TABLES
 */
char* overdrive_query(ODB db, const char* sql);

/**
 * Full-text search across all string fields in a table.
 * Returns a JSON array of matching documents. Must be freed.
 * Returns NULL on error or no matches.
 */
char* overdrive_search(ODB db, const char* table, const char* text);

/* ═══════════════════════════════════════════════════════════
 * MVCC TRANSACTIONS
 * ═══════════════════════════════════════════════════════════ */

/**
 * Begin a new MVCC transaction.
 * isolation_level: 0=ReadUncommitted, 1=ReadCommitted (default), 2=RepeatableRead, 3=Serializable
 * Returns transaction ID (> 0) on success, 0 on error.
 */
uint64_t overdrive_begin_transaction(ODB db, int isolation_level);

/**
 * Commit a transaction. Returns 0 on success, -1 on error.
 */
int overdrive_commit_transaction(ODB db, uint64_t txn_id);

/**
 * Abort (rollback) a transaction. Returns 0 on success, -1 on error.
 */
int overdrive_abort_transaction(ODB db, uint64_t txn_id);

/* ═══════════════════════════════════════════════════════════
 * RAM ENGINE (v1.4)
 * ═══════════════════════════════════════════════════════════ */

/**
 * Persist the current RAM database to a snapshot file on disk.
 * Returns 0 on success, -1 on error.
 */
int overdrive_snapshot(ODB db, const char* snapshot_path);

/**
 * Load a previously saved snapshot into the current database handle.
 * Returns 0 on success, -1 on error.
 */
int overdrive_restore(ODB db, const char* snapshot_path);

/**
 * Get current RAM consumption statistics.
 * Returns JSON: {"bytes":N,"mb":N.N,"limit_bytes":N,"percent":N.N}
 * Must be freed. Returns NULL on error.
 */
char* overdrive_memory_usage(ODB db);

/**
 * Set the memory limit for the RAM store in bytes.
 * Returns 0 on success, -1 on error.
 */
int overdrive_set_memory_limit(ODB db, uint64_t max_bytes);

/* ═══════════════════════════════════════════════════════════
 * WATCHDOG — FILE INTEGRITY MONITORING (v1.4)
 * ═══════════════════════════════════════════════════════════ */

/**
 * Inspect a .odb file for integrity, size, and modification status.
 * Does NOT require an open database handle — pass the file path directly.
 * Completes in < 100ms for files under 1GB.
 *
 * Returns JSON:
 * {
 *   "file_path": "...",
 *   "file_size_bytes": N,
 *   "last_modified": N,          // Unix timestamp
 *   "integrity_status": "valid" | "corrupted" | "missing",
 *   "corruption_details": "..." | null,
 *   "page_count": N,
 *   "magic_valid": true | false
 * }
 * Must be freed. Returns NULL on error.
 */
char* overdrive_watchdog(const char* path);

/* ═══════════════════════════════════════════════════════════
 * INTEGRITY & STATISTICS
 * ═══════════════════════════════════════════════════════════ */

/**
 * Verify database integrity (B-Tree consistency, page checksums, MVCC chains).
 * Returns JSON: {"valid":bool,"pages_checked":N,"tables_verified":N,"issues":[...]}
 * Must be freed. Returns NULL on error.
 */
char* overdrive_verify_integrity(ODB db);

/**
 * Get extended database statistics.
 * Returns JSON: {"tables":N,"total_records":N,"file_size_bytes":N,"path":"...","sdk_version":"..."}
 * Must be freed. Returns NULL on error.
 */
char* overdrive_stats(ODB db);

/* ═══════════════════════════════════════════════════════════
 * ERROR HANDLING & UTILITY
 * ═══════════════════════════════════════════════════════════ */

/**
 * Get the last error message. Do NOT free this pointer.
 * Returns NULL if no error. Valid until the next API call on this thread.
 */
const char* overdrive_last_error(void);

/**
 * Get structured error details as JSON. Do NOT free this pointer. (v1.4)
 * Returns JSON: {"code":"ODB-AUTH-001","message":"...","context":"...","suggestions":[...],"doc_link":"..."}
 * Returns NULL if no error.
 */
const char* overdrive_get_error_details(void);

/**
 * Free a string returned by overdrive_* functions.
 * MUST be called on every non-NULL char* returned by:
 *   insert, get, query, search, list_tables, memory_usage, watchdog,
 *   verify_integrity, stats
 */
void overdrive_free_string(char* s);

/**
 * Enable or disable auto-table creation on first insert. (v1.4)
 * enabled: 1 = on (default), 0 = off
 * Returns 0 on success, -1 on error.
 */
int overdrive_set_auto_create_tables(ODB db, int enabled);

#ifdef __cplusplus
}
#endif

#endif /* OVERDRIVE_H */
