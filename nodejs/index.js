/**
 * OverDrive InCode SDK — Node.js Wrapper
 * Embeddable document database. Like SQLite for JSON.
 *
 * Usage:
 *   const { OverDrive } = require('overdrive-db');
 *   const db = new OverDrive('myapp.odb');
 *   db.createTable('users');
 *   db.insert('users', { name: 'Alice', age: 30 });
 *   const results = db.query('SELECT * FROM users WHERE age > 25');
 *   db.close();
 */

const ffi = require('ffi-napi');
const ref = require('ref-napi');
const path = require('path');
const os = require('os');

// Find the native library
function findLibrary() {
    const platform = os.platform();
    let libName;
    if (platform === 'win32') libName = 'overdrive.dll';
    else if (platform === 'darwin') libName = 'liboverdrive.dylib';
    else libName = 'liboverdrive.so';

    const searchPaths = [
        path.join(__dirname, libName),
        path.join(__dirname, 'lib', libName),
        path.join(__dirname, '..', 'target', 'release', libName),
        path.join(__dirname, '..', '..', 'target', 'release', libName),
    ];

    for (const p of searchPaths) {
        try {
            require('fs').accessSync(p);
            return p;
        } catch { }
    }
    return libName; // Fall back to system path
}

// Load native library
const libPath = findLibrary();
const voidPtr = ref.refType(ref.types.void);

const lib = ffi.Library(libPath, {
    'overdrive_open': [voidPtr, ['string']],
    'overdrive_close': ['void', [voidPtr]],
    'overdrive_sync': ['void', [voidPtr]],
    'overdrive_create_table': ['int', [voidPtr, 'string']],
    'overdrive_drop_table': ['int', [voidPtr, 'string']],
    'overdrive_list_tables': [voidPtr, [voidPtr]],
    'overdrive_table_exists': ['int', [voidPtr, 'string']],
    'overdrive_insert': [voidPtr, [voidPtr, 'string', 'string']],
    'overdrive_get': [voidPtr, [voidPtr, 'string', 'string']],
    'overdrive_update': ['int', [voidPtr, 'string', 'string', 'string']],
    'overdrive_delete': ['int', [voidPtr, 'string', 'string']],
    'overdrive_count': ['int', [voidPtr, 'string']],
    'overdrive_query': [voidPtr, [voidPtr, 'string']],
    'overdrive_search': [voidPtr, [voidPtr, 'string', 'string']],
    'overdrive_last_error': ['string', []],
    'overdrive_free_string': ['void', [voidPtr]],
    'overdrive_version': ['string', []],
});

function readAndFree(ptr) {
    if (ref.isNull(ptr)) return null;
    const str = ptr.readCString();
    lib.overdrive_free_string(ptr);
    return str;
}

function checkError() {
    const err = lib.overdrive_last_error();
    if (err) throw new Error(`OverDrive: ${err}`);
}

class OverDrive {
    /**
     * Open (or create) a database.
     * @param {string} dbPath - Path to the database file
     */
    constructor(dbPath) {
        this._handle = lib.overdrive_open(dbPath);
        if (ref.isNull(this._handle)) {
            checkError();
            throw new Error(`Failed to open database: ${dbPath}`);
        }
        this._path = dbPath;
    }

    /** Close the database. */
    close() {
        if (this._handle && !ref.isNull(this._handle)) {
            lib.overdrive_close(this._handle);
            this._handle = null;
        }
    }

    /** Sync data to disk. */
    sync() {
        this._ensureOpen();
        lib.overdrive_sync(this._handle);
    }

    /** Get database file path. */
    get path() { return this._path; }

    /** Get SDK version. */
    static version() { return lib.overdrive_version(); }

    // ── Tables ─────────────────────────────────

    /** Create a table. */
    createTable(name) {
        this._ensureOpen();
        if (lib.overdrive_create_table(this._handle, name) !== 0) checkError();
    }

    /** Drop a table. */
    dropTable(name) {
        this._ensureOpen();
        if (lib.overdrive_drop_table(this._handle, name) !== 0) checkError();
    }

    /** List all tables. */
    listTables() {
        this._ensureOpen();
        const ptr = lib.overdrive_list_tables(this._handle);
        if (ref.isNull(ptr)) { checkError(); return []; }
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : [];
    }

    /** Check if table exists. */
    tableExists(name) {
        this._ensureOpen();
        return lib.overdrive_table_exists(this._handle, name) === 1;
    }

    // ── CRUD ───────────────────────────────────

    /**
     * Insert a document.
     * @param {string} table - Table name
     * @param {object} doc - JSON document
     * @returns {string} The generated _id
     */
    insert(table, doc) {
        this._ensureOpen();
        const ptr = lib.overdrive_insert(this._handle, table, JSON.stringify(doc));
        if (ref.isNull(ptr)) { checkError(); throw new Error('Insert failed'); }
        return readAndFree(ptr);
    }

    /**
     * Insert multiple documents.
     * @param {string} table
     * @param {object[]} docs
     * @returns {string[]} List of _ids
     */
    insertMany(table, docs) {
        return docs.map(doc => this.insert(table, doc));
    }

    /**
     * Get a document by _id.
     * @returns {object|null}
     */
    get(table, id) {
        this._ensureOpen();
        const ptr = lib.overdrive_get(this._handle, table, id);
        if (ref.isNull(ptr)) return null;
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : null;
    }

    /**
     * Update a document by _id.
     * @returns {boolean} True if updated
     */
    update(table, id, updates) {
        this._ensureOpen();
        const result = lib.overdrive_update(this._handle, table, id, JSON.stringify(updates));
        if (result === -1) checkError();
        return result === 1;
    }

    /**
     * Delete a document by _id.
     * @returns {boolean} True if deleted
     */
    delete(table, id) {
        this._ensureOpen();
        const result = lib.overdrive_delete(this._handle, table, id);
        if (result === -1) checkError();
        return result === 1;
    }

    /**
     * Count documents in a table.
     * @returns {number}
     */
    count(table) {
        this._ensureOpen();
        const result = lib.overdrive_count(this._handle, table);
        if (result === -1) checkError();
        return Math.max(0, result);
    }

    // ── Query ──────────────────────────────────

    /**
     * Execute SQL query.
     * @param {string} sql - SQL query string
     * @returns {object[]} Result rows
     */
    query(sql) {
        this._ensureOpen();
        const ptr = lib.overdrive_query(this._handle, sql);
        if (ref.isNull(ptr)) { checkError(); return []; }
        const result = readAndFree(ptr);
        if (!result) return [];
        const parsed = JSON.parse(result);
        return parsed.rows || [];
    }

    /**
     * Execute SQL query, returning full result with metadata.
     * @returns {{ rows: object[], columns: string[], rows_affected: number }}
     */
    queryFull(sql) {
        this._ensureOpen();
        const ptr = lib.overdrive_query(this._handle, sql);
        if (ref.isNull(ptr)) { checkError(); return { rows: [], columns: [], rows_affected: 0 }; }
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : {};
    }

    /**
     * Full-text search.
     * @returns {object[]}
     */
    search(table, text) {
        this._ensureOpen();
        const ptr = lib.overdrive_search(this._handle, table, text);
        if (ref.isNull(ptr)) return [];
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : [];
    }

    // ── Internal ───────────────────────────────

    _ensureOpen() {
        if (!this._handle || ref.isNull(this._handle)) {
            throw new Error('Database is closed');
        }
    }
}

module.exports = { OverDrive };
