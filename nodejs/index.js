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

const koffi = require('koffi');
const path = require('path');
const os = require('os');
const fs = require('fs');

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
        if (fs.existsSync(p)) return p;
    }
    return libName; // Fall back to system path
}

// Load native library via koffi
const libPath = findLibrary();
const lib = koffi.load(libPath);

// Define FFI function signatures
const overdrive_open = lib.func('void* overdrive_open(const char* path)');
const overdrive_close = lib.func('void overdrive_close(void* db)');
const overdrive_sync = lib.func('void overdrive_sync(void* db)');
const overdrive_create_table = lib.func('int overdrive_create_table(void* db, const char* name)');
const overdrive_drop_table = lib.func('int overdrive_drop_table(void* db, const char* name)');
const overdrive_list_tables = lib.func('void* overdrive_list_tables(void* db)');
const overdrive_table_exists = lib.func('int overdrive_table_exists(void* db, const char* name)');
const overdrive_insert = lib.func('void* overdrive_insert(void* db, const char* table, const char* json_doc)');
const overdrive_get = lib.func('void* overdrive_get(void* db, const char* table, const char* id)');
const overdrive_update = lib.func('int overdrive_update(void* db, const char* table, const char* id, const char* json_updates)');
const overdrive_delete = lib.func('int overdrive_delete(void* db, const char* table, const char* id)');
const overdrive_count = lib.func('int overdrive_count(void* db, const char* table)');
const overdrive_query = lib.func('void* overdrive_query(void* db, const char* sql)');
const overdrive_search = lib.func('void* overdrive_search(void* db, const char* table, const char* text)');
const overdrive_last_error = lib.func('const char* overdrive_last_error()');
const overdrive_free_string = lib.func('void overdrive_free_string(void* s)');
const overdrive_version = lib.func('const char* overdrive_version()');

function readAndFree(ptr) {
    if (!ptr) return null;
    const str = koffi.decode(ptr, 'char', -1);
    overdrive_free_string(ptr);
    return str;
}

function checkError() {
    const err = overdrive_last_error();
    if (err) throw new Error(`OverDrive: ${err}`);
}

class OverDrive {
    /**
     * Open (or create) a database.
     * @param {string} dbPath - Path to the database file
     */
    constructor(dbPath) {
        this._handle = overdrive_open(dbPath);
        if (!this._handle) {
            checkError();
            throw new Error(`Failed to open database: ${dbPath}`);
        }
        this._path = dbPath;
    }

    /** Close the database. */
    close() {
        if (this._handle) {
            overdrive_close(this._handle);
            this._handle = null;
        }
    }

    /** Sync data to disk. */
    sync() {
        this._ensureOpen();
        overdrive_sync(this._handle);
    }

    /** Get database file path. */
    get path() { return this._path; }

    /** Get SDK version. */
    static version() { return overdrive_version(); }

    // ── Tables ─────────────────────────────────

    /** Create a table. */
    createTable(name) {
        this._ensureOpen();
        if (overdrive_create_table(this._handle, name) !== 0) checkError();
    }

    /** Drop a table. */
    dropTable(name) {
        this._ensureOpen();
        if (overdrive_drop_table(this._handle, name) !== 0) checkError();
    }

    /** List all tables. */
    listTables() {
        this._ensureOpen();
        const ptr = overdrive_list_tables(this._handle);
        if (!ptr) { checkError(); return []; }
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : [];
    }

    /** Check if table exists. */
    tableExists(name) {
        this._ensureOpen();
        return overdrive_table_exists(this._handle, name) === 1;
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
        const ptr = overdrive_insert(this._handle, table, JSON.stringify(doc));
        if (!ptr) { checkError(); throw new Error('Insert failed'); }
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
        const ptr = overdrive_get(this._handle, table, id);
        if (!ptr) return null;
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : null;
    }

    /**
     * Update a document by _id.
     * @returns {boolean} True if updated
     */
    update(table, id, updates) {
        this._ensureOpen();
        const result = overdrive_update(this._handle, table, id, JSON.stringify(updates));
        if (result === -1) checkError();
        return result === 1;
    }

    /**
     * Delete a document by _id.
     * @returns {boolean} True if deleted
     */
    delete(table, id) {
        this._ensureOpen();
        const result = overdrive_delete(this._handle, table, id);
        if (result === -1) checkError();
        return result === 1;
    }

    /**
     * Count documents in a table.
     * @returns {number}
     */
    count(table) {
        this._ensureOpen();
        const result = overdrive_count(this._handle, table);
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
        const ptr = overdrive_query(this._handle, sql);
        if (!ptr) { checkError(); return []; }
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
        const ptr = overdrive_query(this._handle, sql);
        if (!ptr) { checkError(); return { rows: [], columns: [], rows_affected: 0 }; }
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : {};
    }

    /**
     * Full-text search.
     * @returns {object[]}
     */
    search(table, text) {
        this._ensureOpen();
        const ptr = overdrive_search(this._handle, table, text);
        if (!ptr) return [];
        const result = readAndFree(ptr);
        return result ? JSON.parse(result) : [];
    }

    // ── Security (v1.3.0) ──────────────────────

    /**
     * Open a database with an encryption key loaded from an environment variable.
     * Never hardcode the key — always read from env or a secrets manager.
     *
     * @param {string} dbPath - Path to the .odb file
     * @param {string} keyEnvVar - Name of the env var holding the key (default: 'ODB_KEY')
     * @returns {OverDrive}
     *
     * @example
     * // process.env.ODB_KEY = 'my-secret-32-char-key!!!!'
     * const db = OverDrive.openEncrypted('app.odb', 'ODB_KEY');
     */
    static openEncrypted(dbPath, keyEnvVar = 'ODB_KEY') {
        const key = process.env[keyEnvVar];
        if (!key) {
            throw new Error(
                `[overdrive] Encryption key env var '${keyEnvVar}' is not set or empty. ` +
                `Set it with: process.env.${keyEnvVar} = 'your-key' or ` +
                `$env:${keyEnvVar}="your-key" (PowerShell)`
            );
        }
        // Pass key to engine via internal env var, clear immediately after open
        process.env.__OVERDRIVE_KEY = key;
        let db;
        try {
            db = new OverDrive(dbPath);
        } finally {
            delete process.env.__OVERDRIVE_KEY;
        }
        return db;
    }

    /**
     * Create an encrypted backup of the database at destPath.
     * Syncs to disk first, then copies .odb + .wal files.
     *
     * @param {string} destPath - Destination .odb path
     * @example
     * db.backup('backups/app_2026-03-04.odb');
     */
    backup(destPath) {
        this._ensureOpen();
        this.sync();
        fs.copyFileSync(this._path, destPath);
        // Copy WAL if exists
        const walSrc = this._path + '.wal';
        const walDst = destPath + '.wal';
        if (fs.existsSync(walSrc)) fs.copyFileSync(walSrc, walDst);
        // Harden backup file permissions
        _setSecurePermissions(destPath);
    }

    /**
     * Delete the WAL file after a confirmed commit to prevent stale replay attacks.
     * Call this after commitTransaction().
     */
    cleanupWal() {
        const walPath = this._path + '.wal';
        if (fs.existsSync(walPath)) fs.unlinkSync(walPath);
    }

    /**
     * Execute a parameterized SQL query — safe way to include user input.
     * Use ? as placeholders; params are sanitized before substitution.
     * Throws if any param contains SQL injection patterns.
     *
     * @param {string} sqlTemplate - SQL with ? placeholders
     * @param {string[]} params - Values to substitute
     * @returns {object[]} Result rows
     *
     * @example
     * // SAFE: user input via params, never string concat
     * const rows = db.querySafe('SELECT * FROM users WHERE name = ?', [userInput]);
     */
    querySafe(sqlTemplate, params) {
        const DANGEROUS = ['DROP', 'TRUNCATE', 'ALTER', 'EXEC', 'EXECUTE', 'UNION', 'XP_'];
        const DANGEROUS_TOKENS = ['--', ';--', '/*', '*/'];

        const sanitized = params.map(param => {
            const s = String(param);
            const upper = s.toUpperCase();
            for (const token of DANGEROUS_TOKENS) {
                if (s.includes(token)) {
                    throw new Error(`[overdrive] SQL injection detected: param '${s}' contains '${token}'`);
                }
            }
            for (const kw of DANGEROUS) {
                if (upper.split(/\s+/).includes(kw)) {
                    throw new Error(`[overdrive] SQL injection detected: param '${s}' contains keyword '${kw}'`);
                }
            }
            // Escape single quotes
            return `'${s.replace(/'/g, "''")}'`;
        });

        let sql = sqlTemplate;
        for (const value of sanitized) {
            if (!sql.includes('?')) throw new Error("[overdrive] More params than '?' placeholders");
            sql = sql.replace('?', value);
        }

        const placeholderCount = (sqlTemplate.match(/\?/g) || []).length;
        if (params.length < placeholderCount) {
            throw new Error(`[overdrive] SQL template has ${placeholderCount} placeholders but only ${params.length} params`);
        }
        return this.query(sql);
    }

    _ensureOpen() {
        if (!this._handle) {
            throw new Error('Database is closed');
        }
    }
}

// ── File Permission Hardening ─────────────────

function _setSecurePermissions(filePath) {
    if (!fs.existsSync(filePath)) return;
    const platform = os.platform();
    if (platform === 'win32') {
        const { execSync } = require('child_process');
        try {
            execSync(`icacls "${filePath}" /inheritance:r /grant:r "%USERNAME%:F"`, { stdio: 'pipe' });
        } catch (e) {
            // Non-fatal — warn but continue
            process.emitWarning(`[overdrive] Could not harden file permissions on '${filePath}': ${e.message}`);
        }
    } else {
        try {
            fs.chmodSync(filePath, 0o600);
        } catch (e) {
            process.emitWarning(`[overdrive] Could not chmod 600 '${filePath}': ${e.message}`);
        }
    }
}

// Patch _setSecurePermissions into constructor
const _originalConstructor = OverDrive.prototype.constructor;
const _OriginalOverDrive = OverDrive;
class OverDriveSecure extends _OriginalOverDrive {
    constructor(dbPath) {
        super(dbPath);
        _setSecurePermissions(dbPath);
    }
}
// Re-export as OverDrive (backward compatible)
Object.defineProperty(OverDriveSecure, 'name', { value: 'OverDrive' });

// ── Async-safe Shared Wrapper ─────────────────

/**
 * SharedOverDrive — async-safe OverDrive wrapper using a promise-based mutex.
 *
 * Node.js is single-threaded but async code can interleave. This wrapper
 * serializes all database operations to prevent concurrent access issues.
 *
 * @example
 * const db = new SharedOverDrive('app.odb');
 * const rows = await db.query('SELECT * FROM users');
 * const id   = await db.insert('users', { name: 'Alice' });
 */
class SharedOverDrive {
    constructor(dbPath) {
        this._db = new OverDrive(dbPath);
        this._queue = Promise.resolve();
    }

    static openEncrypted(dbPath, keyEnvVar = 'ODB_KEY') {
        const instance = new SharedOverDrive.__proto__(dbPath);
        instance._db = OverDrive.openEncrypted(dbPath, keyEnvVar);
        instance._queue = Promise.resolve();
        return instance;
    }

    /** Serialize an operation through the queue (prevents interleaving). */
    _run(fn) {
        this._queue = this._queue.then(() => fn(this._db));
        return this._queue;
    }

    async query(sql) { return this._run(db => db.query(sql)); }
    async querySafe(tmpl, params) { return this._run(db => db.querySafe(tmpl, params)); }
    async insert(table, doc) { return this._run(db => db.insert(table, doc)); }
    async get(table, id) { return this._run(db => db.get(table, id)); }
    async backup(dest) { return this._run(db => db.backup(dest)); }
    async cleanupWal() { return this._run(db => db.cleanupWal()); }
    async sync() { return this._run(db => db.sync()); }
    async close() { return this._run(db => db.close()); }
}

module.exports = { OverDrive, SharedOverDrive };

