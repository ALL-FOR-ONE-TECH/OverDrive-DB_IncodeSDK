/**
 * OverDrive InCode SDK — C/C++ Header
 *
 * Embeddable document database. Like SQLite for JSON.
 *
 * Link against: overdrive.dll (Windows) / liboverdrive.so (Linux) / liboverdrive.dylib (macOS)
 *
 * Usage:
 *   ODB* db = overdrive_open("myapp.odb");
 *   overdrive_create_table(db, "users");
 *   char* id = overdrive_insert(db, "users", "{\"name\":\"Alice\"}");
 *   overdrive_free_string(id);
 *   overdrive_close(db);
 */

#ifndef OVERDRIVE_H
#define OVERDRIVE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque database handle.
 */
typedef void* ODB;

/* ── Lifecycle ──────────────────────────────── */

/**
 * Open (or create) a database. Returns NULL on error.
 * Check overdrive_last_error() for details.
 */
ODB overdrive_open(const char* path);

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

/* ── Tables ─────────────────────────────────── */

/**
 * Create a new table. Returns 0 on success, -1 on error.
 */
int overdrive_create_table(ODB db, const char* name);

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
 * Check if a table exists. Returns 1 if exists, 0 if not.
 */
int overdrive_table_exists(ODB db, const char* name);

/* ── CRUD ───────────────────────────────────── */

/**
 * Insert a JSON document into a table.
 * Returns the generated _id string. Must be freed with overdrive_free_string().
 * Returns NULL on error.
 */
char* overdrive_insert(ODB db, const char* table, const char* json_doc);

/**
 * Get a document by _id.
 * Returns JSON string. Must be freed with overdrive_free_string().
 * Returns NULL if not found.
 */
char* overdrive_get(ODB db, const char* table, const char* id);

/**
 * Update document fields by _id.
 * json_updates is a JSON object with fields to merge.
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

/* ── Query ──────────────────────────────────── */

/**
 * Execute an SQL query.
 * Returns a JSON result string. Must be freed with overdrive_free_string().
 * Result format: {"rows": [...], "columns": [...], "rows_affected": N}
 * Returns NULL on error.
 */
char* overdrive_query(ODB db, const char* sql);

/**
 * Full-text search across all string fields in a table.
 * Returns a JSON array of matching documents. Must be freed with overdrive_free_string().
 * Returns NULL on error or no matches.
 */
char* overdrive_search(ODB db, const char* table, const char* text);

/* ── Utility ────────────────────────────────── */

/**
 * Get the last error message. Do NOT free this pointer.
 * Returns NULL if no error.
 */
const char* overdrive_last_error(void);

/**
 * Free a string returned by overdrive_* functions.
 * Must be called on every non-NULL char* returned by insert, get, query, search, list_tables.
 */
void overdrive_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif /* OVERDRIVE_H */
