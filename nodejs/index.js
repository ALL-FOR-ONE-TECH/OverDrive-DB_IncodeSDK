/**
 * OverDrive InCode SDK — Node.js Wrapper
 * Embeddable document database. Like SQLite for JSON.
 *
 * Usage (v1.4 simplified API):
 *   const { OverDrive } = require('overdrive-db');
 *   const db = OverDrive.open('myapp.odb');
 *   db.insert('users', { name: 'Alice', age: 30 });  // table auto-created
 *   const results = db.query('SELECT * FROM users WHERE age > 25');
 *   db.close();
 *
 * Password-protected:
 *   const db = OverDrive.open('secure.odb', { password: 'my-secret-pass' });
 *
 * RAM engine:
 *   const db = OverDrive.open('cache.odb', { engine: 'RAM' });
 *
 * Legacy API (still fully supported):
 *   const db = new OverDrive('myapp.odb');
 *   db.createTable('users');
 *   db.insert('users', { name: 'Alice', age: 30 });
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

// MVCC Transactions — loaded lazily (not in all native lib versions)
// v1.4: New FFI functions — loaded lazily to allow graceful degradation
// when running against an older native library that doesn't have them yet.
// NOTE: These wrapper functions use plain Error for the "not available" case
// because the OverDriveError subclasses are defined later in the file.
// At call time, the error classes are available and will be used by callers.
const _v14 = {};

function _loadV14Func(name, sig) {
    if (_v14[name] !== undefined) return _v14[name];
    try {
        _v14[name] = lib.func(sig);
    } catch (_) {
        _v14[name] = null; // not available in this native lib version
    }
    return _v14[name];
}

function overdrive_open_with_password(p, password) {
    const fn = _loadV14Func('overdrive_open_with_password',
        'void* overdrive_open_with_password(const char* path, const char* password)');
    if (!fn) {
        const e = new Error('overdrive_open_with_password not available in this native library version. Please update the native library.');
        e._odbCode = 'ODB-FFI-001';
        throw e;
    }
    return fn(p, password);
}

function overdrive_open_with_engine(p, engine, opts) {
    const fn = _loadV14Func('overdrive_open_with_engine',
        'void* overdrive_open_with_engine(const char* path, const char* engine, const char* options_json)');
    if (!fn) {
        const e = new Error('overdrive_open_with_engine not available in this native library version. Please update the native library.');
        e._odbCode = 'ODB-FFI-001';
        throw e;
    }
    return fn(p, engine, opts);
}

function overdrive_set_auto_create_tables(db, enabled) {
    const fn = _loadV14Func('overdrive_set_auto_create_tables',
        'int overdrive_set_auto_create_tables(void* db, int enabled)');
    if (!fn) return 0; // graceful no-op
    return fn(db, enabled);
}

function overdrive_get_error_details() {
    const fn = _loadV14Func('overdrive_get_error_details',
        'const char* overdrive_get_error_details()');
    if (!fn) return null;
    return fn();
}

function overdrive_create_table_with_engine(db, name, engine) {
    const fn = _loadV14Func('overdrive_create_table_with_engine',
        'int overdrive_create_table_with_engine(void* db, const char* name, const char* engine)');
    if (!fn) {
        const e = new Error('overdrive_create_table_with_engine not available in this native library version.');
        e._odbCode = 'ODB-FFI-001';
        throw e;
    }
    return fn(db, name, engine);
}

function overdrive_snapshot(db, p) {
    const fn = _loadV14Func('overdrive_snapshot',
        'int overdrive_snapshot(void* db, const char* path)');
    if (!fn) {
        const e = new Error('overdrive_snapshot not available in this native library version.');
        e._odbCode = 'ODB-FFI-001';
        throw e;
    }
    return fn(db, p);
}

function overdrive_restore(db, p) {
    const fn = _loadV14Func('overdrive_restore',
        'int overdrive_restore(void* db, const char* path)');
    if (!fn) {
        const e = new Error('overdrive_restore not available in this native library version.');
        e._odbCode = 'ODB-FFI-001';
        throw e;
    }
    return fn(db, p);
}

function overdrive_memory_usage(db) {
    const fn = _loadV14Func('overdrive_memory_usage',
        'void* overdrive_memory_usage(void* db)');
    if (!fn) {
        const e = new Error('overdrive_memory_usage not available in this native library version.');
        e._odbCode = 'ODB-FFI-001';
        throw e;
    }
    return fn(db);
}

function overdrive_watchdog(p) {
    const fn = _loadV14Func('overdrive_watchdog',
        'void* overdrive_watchdog(const char* path)');
    if (!fn) {
        const e = new Error('overdrive_watchdog not available in this native library version.');
        e._odbCode = 'ODB-FFI-001';
        throw e;
    }
    return fn(p);
}

function overdrive_begin_transaction(db, isolation) {
    const fn = _loadV14Func('overdrive_begin_transaction',
        'uint64_t overdrive_begin_transaction(void* db, int isolation)');
    if (!fn) {
        const e = new Error('overdrive_begin_transaction not available in this native library version.');
        e._odbCode = 'ODB-TXN-001';
        throw e;
    }
    return fn(db, isolation);
}

function overdrive_commit_transaction(db, txnId) {
    const fn = _loadV14Func('overdrive_commit_transaction',
        'int overdrive_commit_transaction(void* db, uint64_t txn_id)');
    if (!fn) {
        const e = new Error('overdrive_commit_transaction not available in this native library version.');
        e._odbCode = 'ODB-TXN-001';
        throw e;
    }
    return fn(db, txnId);
}

function overdrive_abort_transaction(db, txnId) {
    const fn = _loadV14Func('overdrive_abort_transaction',
        'int overdrive_abort_transaction(void* db, uint64_t txn_id)');
    if (!fn) {
        const e = new Error('overdrive_abort_transaction not available in this native library version.');
        e._odbCode = 'ODB-TXN-001';
        throw e;
    }
    return fn(db, txnId);
}

function readAndFree(ptr) {
    if (!ptr) return null;
    const str = koffi.decode(ptr, 'char', -1);
    overdrive_free_string(ptr);
    return str;
}

// ── Error Hierarchy (Task 18) ─────────────────

/**
 * Base class for all OverDrive SDK errors.
 */
class OverDriveError extends Error {
    constructor(message, opts) {
        const o = opts || {};
        const code = o.code || '';
        const context = o.context || '';
        const suggestions = o.suggestions || [];
        const docLink = o.docLink || '';
        const parts = [code ? ('Error ' + code + ': ' + message) : message];
        if (context) parts.push('Context: ' + context);
        if (suggestions && suggestions.length) {
            parts.push('Suggestions:\n' + suggestions.map(function(s) { return '  \u2022 ' + s; }).join('\n'));
        }
        if (docLink) parts.push('For more help: ' + docLink);
        super(parts.join('\n'));
        this.name = 'OverDriveError';
        this.code = code;
        this.context = context;
        this.suggestions = suggestions;
        this.docLink = docLink;
    }
}

class AuthenticationError extends OverDriveError {
    constructor(message, opts) { super(message, opts); this.name = 'AuthenticationError'; }
}
class TableError extends OverDriveError {
    constructor(message, opts) { super(message, opts); this.name = 'TableError'; }
}
class QueryError extends OverDriveError {
    constructor(message, opts) { super(message, opts); this.name = 'QueryError'; }
}
class TransactionError extends OverDriveError {
    constructor(message, opts) { super(message, opts); this.name = 'TransactionError'; }
}
class OverDriveIOError extends OverDriveError {
    constructor(message, opts) { super(message, opts); this.name = 'OverDriveIOError'; }
}
class FFIError extends OverDriveError {
    constructor(message, opts) { super(message, opts); this.name = 'FFIError'; }
}

const _ERROR_CLASS_MAP = {
    'ODB-AUTH': AuthenticationError,
    'ODB-TABLE': TableError,
    'ODB-QUERY': QueryError,
    'ODB-TXN': TransactionError,
    'ODB-IO': OverDriveIOError,
    'ODB-FFI': FFIError,
};

function _makeError(message, opts) {
    const o = opts || {};
    const code = o.code || '';
    let cls = OverDriveError;
    if (code) {
        const parts = code.split('-');
        if (parts.length >= 3) {
            const prefix = parts[0] + '-' + parts[1];
            cls = _ERROR_CLASS_MAP[prefix] || OverDriveError;
        }
    }
    return new cls(message, o);
}

/**
 * Check for and throw the last error from the native library.
 * Tries overdrive_get_error_details() first for structured JSON,
 * falls back to overdrive_last_error() for a plain string.
 */
function checkErrorStructured() {
    try {
        const detailsRaw = overdrive_get_error_details();
        if (detailsRaw) {
            try {
                const data = JSON.parse(detailsRaw);
                const code = data.code || '';
                const msg = data.message || '';
                const context = data.context || '';
                const suggestions = data.suggestions || [];
                const docLink = data.doc_link || '';
                if (msg || code) {
                    throw _makeError(msg, { code, context, suggestions, docLink });
                }
            } catch (e) {
                if (e instanceof OverDriveError) throw e;
            }
        }
    } catch (e) {
        if (e instanceof OverDriveError) throw e;
    }
    const err = overdrive_last_error();
    if (err) throw new OverDriveError(err);
}

class OverDrive {
    /**
     * Open (or create) a database.
     * @param {string} dbPath - Path to the database file
     */
    constructor(dbPath) {
        this._handle = overdrive_open(dbPath);
        if (!this._handle) {
            checkErrorStructured();
            throw new OverDriveError('Failed to open database: ' + dbPath);
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

    /**
     * Create a table.
     * @param {string} name - Table name
     * @param {{ engine?: string }} [options={}] - Options
     */
    createTable(name, options) {
        this._ensureOpen();
        const engine = (options && options.engine) ? options.engine : 'Disk';
        let result;
        if (engine === 'Disk') {
            result = overdrive_create_table(this._handle, name);
        } else {
            result = overdrive_create_table_with_engine(this._handle, name, engine);
        }
        if (result !== 0) checkErrorStructured();
    }

    /** Drop a table. */
    dropTable(name) {
        this._ensureOpen();
        if (overdrive_drop_table(this._handle, name) !== 0) checkErrorStructured();
    }

    /** List all tables. */
    listTables() {
        this._ensureOpen();
        const ptr = overdrive_list_tables(this._handle);
        if (!ptr) { checkErrorStructured(); return []; }
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
     * @param {string} table
     * @param {object} doc
     * @returns {string} The generated _id
     */
    insert(table, doc) {
        this._ensureOpen();
        const ptr = overdrive_insert(this._handle, table, JSON.stringify(doc));
        if (!ptr) { checkErrorStructured(); throw new OverDriveError('Insert failed'); }
        return readAndFree(ptr);
    }

    /**
     * Insert multiple documents.
     * @param {string} table
     * @param {object[]} docs
     * @returns {string[]}
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
     * @returns {boolean}
     */
    update(table, id, updates) {
        this._ensureOpen();
        const result = overdrive_update(this._handle, table, id, JSON.stringify(updates));
        if (result === -1) checkErrorStructured();
        return result === 1;
    }

    /**
     * Delete a document by _id.
     * @returns {boolean}
     */
    delete(table, id) {
        this._ensureOpen();
        const result = overdrive_delete(this._handle, table, id);
        if (result === -1) checkErrorStructured();
        return result === 1;
    }

    /**
     * Count documents in a table.
     * @returns {number}
     */
    count(table) {
        this._ensureOpen();
        const result = overdrive_count(this._handle, table);
        if (result === -1) checkErrorStructured();
        return Math.max(0, result);
    }

    // ── Query ──────────────────────────────────

    /**
     * Execute SQL query.
     * @param {string} sql
     * @returns {object[]}
     */
    query(sql) {
        this._ensureOpen();
        const ptr = overdrive_query(this._handle, sql);
        if (!ptr) { checkErrorStructured(); return []; }
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
        if (!ptr) { checkErrorStructured(); return { rows: [], columns: [], rows_affected: 0 }; }
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
     * @param {string} dbPath
     * @param {string} [keyEnvVar='ODB_KEY']
     * @returns {OverDrive}
     */
    static openEncrypted(dbPath, keyEnvVar) {
        const envVar = keyEnvVar || 'ODB_KEY';
        const key = process.env[envVar];
        if (!key) {
            throw new OverDriveError(
                "Encryption key env var '" + envVar + "' is not set or empty. " +
                "Set it with: process.env." + envVar + " = 'your-key' or " +
                "$env:" + envVar + "=\"your-key\" (PowerShell)"
            );
        }
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
     * @param {string} destPath
     */
    backup(destPath) {
        this._ensureOpen();
        this.sync();
        fs.copyFileSync(this._path, destPath);
        const walSrc = this._path + '.wal';
        const walDst = destPath + '.wal';
        if (fs.existsSync(walSrc)) fs.copyFileSync(walSrc, walDst);
        _setSecurePermissions(destPath);
    }

    /** Delete the WAL file after a confirmed commit. */
    cleanupWal() {
        const walPath = this._path + '.wal';
        if (fs.existsSync(walPath)) fs.unlinkSync(walPath);
    }

    /**
     * Execute a parameterized SQL query — safe way to include user input.
     * @param {string} sqlTemplate
     * @param {string[]} params
     * @returns {object[]}
     */
    querySafe(sqlTemplate, params) {
        const DANGEROUS = ['DROP', 'TRUNCATE', 'ALTER', 'EXEC', 'EXECUTE', 'UNION', 'XP_'];
        const DANGEROUS_TOKENS = ['--', ';--', '/*', '*/'];

        const sanitized = params.map(param => {
            const s = String(param);
            const upper = s.toUpperCase();
            for (const token of DANGEROUS_TOKENS) {
                if (s.includes(token)) {
                    throw new OverDriveError("SQL injection detected: param '" + s + "' contains '" + token + "'");
                }
            }
            for (const kw of DANGEROUS) {
                if (upper.split(/\s+/).includes(kw)) {
                    throw new OverDriveError("SQL injection detected: param '" + s + "' contains keyword '" + kw + "'");
                }
            }
            return "'" + s.replace(/'/g, "''") + "'";
        });

        let sql = sqlTemplate;
        for (const value of sanitized) {
            if (!sql.includes('?')) throw new OverDriveError("More params than '?' placeholders");
            sql = sql.replace('?', value);
        }

        const placeholderCount = (sqlTemplate.match(/\?/g) || []).length;
        if (params.length < placeholderCount) {
            throw new OverDriveError('SQL template has ' + placeholderCount + ' placeholders but only ' + params.length + ' params');
        }
        return this.query(sql);
    }

    // ── Simplified API (Task 13) ───────────────

    /**
     * Open or create a database — the simplified v1.4 entry point.
     *
     * Routing logic (Task 13.2, 13.3):
     *   - No password + engine="Disk"  → overdrive_open() (existing constructor path)
     *   - Password provided            → overdrive_open_with_password()
     *   - engine != "Disk"             → overdrive_open_with_engine()
     *
     * After opening, overdrive_set_auto_create_tables() is called when
     * autoCreateTables differs from the default (Task 13.4).
     *
     * @param {string} [dbPath='./app.odb']
     * @param {object} [options={}]
     * @param {string} [options.password] - Optional password (min 8 chars). Calls overdrive_open_with_password() FFI.
     * @param {string} [options.engine='Disk'] - "Disk"|"RAM"|"Vector"|"Time-Series"|"Graph"|"Streaming"
     * @param {boolean} [options.autoCreateTables=true] - Calls overdrive_set_auto_create_tables() FFI after open.
     * @returns {OverDrive}
     */
    static open(dbPath, options) {
        const p = dbPath || './app.odb';
        const o = options || {};
        const VALID_ENGINES = ['Disk', 'RAM', 'Vector', 'Time-Series', 'Graph', 'Streaming'];
        const engine = o.engine || 'Disk';
        const password = (o.password !== undefined && o.password !== null) ? o.password : null;
        const autoCreateTables = o.autoCreateTables !== false; // default true

        // Validate engine
        if (VALID_ENGINES.indexOf(engine) === -1) {
            throw new OverDriveError(
                "Invalid engine '" + engine + "'. Valid options: " + VALID_ENGINES.sort().join(', ')
            );
        }

        // Validate password length (Task 13.2)
        if (password !== null && password.length < 8) {
            throw new OverDriveError('Password must be at least 8 characters long');
        }

        let handle;

        if (password !== null) {
            // Task 13.2: password provided → call overdrive_open_with_password() FFI
            handle = overdrive_open_with_password(p, password);
        } else if (engine !== 'Disk') {
            // Task 13.3: non-Disk engine → call overdrive_open_with_engine() FFI
            const optionsJson = JSON.stringify({ auto_create_tables: autoCreateTables });
            handle = overdrive_open_with_engine(p, engine, optionsJson);
        } else {
            // Task 13.1/13.5: no password, Disk engine → use existing constructor path
            handle = overdrive_open(p);
        }

        if (!handle) {
            try {
                const detailsRaw = overdrive_get_error_details();
                if (detailsRaw) {
                    const data = JSON.parse(detailsRaw);
                    const code = data.code || '';
                    const msg = data.message || '';
                    const context = data.context || '';
                    const suggestions = data.suggestions || [];
                    const docLink = data.doc_link || '';
                    if (msg || code) throw _makeError(msg, { code, context, suggestions, docLink });
                }
            } catch (e) {
                if (e instanceof OverDriveError) throw e;
            }
            checkErrorStructured();
            throw new OverDriveError('Failed to open database: ' + p);
        }

        const instance = Object.create(OverDrive.prototype);
        instance._handle = handle;
        instance._path = p;

        // Task 13.4: call overdrive_set_auto_create_tables() after opening.
        // For the engine path, auto_create_tables is already passed in options_json.
        // For the password and constructor paths, we call it explicitly.
        if (engine === 'Disk') {
            overdrive_set_auto_create_tables(handle, autoCreateTables ? 1 : 0);
        }

        _setSecurePermissions(p);
        return instance;
    }

    // ── RAM Engine Methods (Task 14) ───────────

    /**
     * Persist the current RAM database to a snapshot file.
     * @param {string} snapshotPath
     */
    snapshot(snapshotPath) {
        this._ensureOpen();
        const result = overdrive_snapshot(this._handle, snapshotPath);
        if (result !== 0) checkErrorStructured();
    }

    /**
     * Load a previously saved snapshot into the current database handle.
     * @param {string} snapshotPath
     */
    restore(snapshotPath) {
        this._ensureOpen();
        const result = overdrive_restore(this._handle, snapshotPath);
        if (result !== 0) checkErrorStructured();
    }

    /**
     * Return current RAM consumption statistics.
     * @returns {{ bytes: number, mb: number, limit_bytes: number, percent: number }}
     */
    memoryUsage() {
        this._ensureOpen();
        const ptr = overdrive_memory_usage(this._handle);
        if (!ptr) {
            checkErrorStructured();
            return { bytes: 0, mb: 0, limit_bytes: 0, percent: 0 };
        }
        const result = readAndFree(ptr);
        if (!result) return { bytes: 0, mb: 0, limit_bytes: 0, percent: 0 };
        const data = JSON.parse(result);
        return {
            bytes: Number(data.bytes || 0),
            mb: Number(data.mb || 0),
            limit_bytes: Number(data.limit_bytes || 0),
            percent: Number(data.percent || 0),
        };
    }

    // ── Watchdog (Task 15) ─────────────────────

    /**
     * Inspect a .odb file for integrity, size, and modification status.
     * Static method — does not require an open database handle.
     *
     * @param {string} filePath
     * @returns {WatchdogReport}
     */
    static watchdog(filePath) {
        const ptr = overdrive_watchdog(filePath);
        if (!ptr) {
            checkErrorStructured();
            throw new OverDriveError('watchdog() returned NULL for path: ' + filePath);
        }
        const raw = readAndFree(ptr);
        if (!raw) throw new OverDriveError('watchdog() returned empty response for path: ' + filePath);
        const data = JSON.parse(raw);
        return {
            filePath: data.file_path || filePath,
            fileSizeBytes: Number(data.file_size_bytes || 0),
            lastModified: Number(data.last_modified || 0),
            integrityStatus: data.integrity_status || 'missing',
            corruptionDetails: data.corruption_details || null,
            pageCount: Number(data.page_count || 0),
            magicValid: Boolean(data.magic_valid),
        };
    }

    // ── MVCC Transactions ──────────────────────

    static get READ_UNCOMMITTED() { return 0; }
    static get READ_COMMITTED() { return 1; }
    static get REPEATABLE_READ() { return 2; }
    static get SERIALIZABLE() { return 3; }

    /**
     * Begin a new MVCC transaction.
     * @param {number} [isolation=1]
     * @returns {number} Transaction ID
     */
    beginTransaction(isolation) {
        this._ensureOpen();
        const iso = (isolation !== undefined) ? isolation : 1;
        const txnId = overdrive_begin_transaction(this._handle, iso);
        if (!txnId) {
            checkErrorStructured();
            throw new TransactionError('Failed to begin transaction');
        }
        return txnId;
    }

    /**
     * Commit a transaction.
     * @param {number} txnId
     */
    commitTransaction(txnId) {
        this._ensureOpen();
        const result = overdrive_commit_transaction(this._handle, txnId);
        if (result !== 0) checkErrorStructured();
    }

    /**
     * Abort (rollback) a transaction.
     * @param {number} txnId
     */
    abortTransaction(txnId) {
        this._ensureOpen();
        const result = overdrive_abort_transaction(this._handle, txnId);
        if (result !== 0) checkErrorStructured();
    }

    // ── Transaction Callback Pattern (Task 16) ─

    /**
     * Dual-mode transaction helper.
     *
     * Callback pattern (auto-commit/rollback):
     *   const result = await db.transaction(async (txn) => {
     *     db.insert('users', { name: 'Alice' });
     *     return 'done';
     *   });
     *
     * No-callback pattern (returns transaction context object):
     *   const txn = db.transaction();
     *   // txn.id, txn.commit(), txn.abort()
     *
     * @param {Function|undefined} [callback]
     * @param {number} [isolation=OverDrive.READ_COMMITTED]
     * @returns {Promise<any>|any|object}
     */
    transaction(callback, isolation) {
        const iso = (isolation !== undefined) ? isolation : OverDrive.READ_COMMITTED;

        if (typeof callback === 'function') {
            const txnId = this.beginTransaction(iso);
            let resultPromise;
            try {
                resultPromise = callback(txnId);
            } catch (err) {
                try { this.abortTransaction(txnId); } catch (_) {}
                throw err;
            }
            // Support async callbacks
            if (resultPromise && typeof resultPromise.then === 'function') {
                return resultPromise.then(
                    (value) => {
                        this.commitTransaction(txnId);
                        return value;
                    },
                    (err) => {
                        try { this.abortTransaction(txnId); } catch (_) {}
                        throw err;
                    }
                );
            }
            // Synchronous callback
            this.commitTransaction(txnId);
            return resultPromise;
        }

        // No callback — return a transaction context object (backward compat)
        const txnId = this.beginTransaction(iso);
        const self = this;
        return {
            id: txnId,
            commit() { self.commitTransaction(txnId); },
            abort() { self.abortTransaction(txnId); },
        };
    }

    // ── Transaction with Retry (Task 18.5) ─────

    /**
     * Execute a transaction with automatic exponential-backoff retry on TransactionError.
     *
     * @param {Function} callback
     * @param {number} [isolation=OverDrive.READ_COMMITTED]
     * @param {number} [maxRetries=3]
     * @returns {Promise<any>}
     */
    async transactionWithRetry(callback, isolation, maxRetries) {
        const iso = (isolation !== undefined) ? isolation : OverDrive.READ_COMMITTED;
        const retries = (maxRetries !== undefined) ? maxRetries : 3;
        let lastErr;
        for (let attempt = 0; attempt < retries; attempt++) {
            try {
                return await this.transaction(callback, iso);
            } catch (err) {
                if (err instanceof TransactionError) {
                    lastErr = err;
                    if (attempt < retries - 1) {
                        const delay = Math.min(100 * Math.pow(2, attempt), 2000);
                        await new Promise(resolve => setTimeout(resolve, delay));
                    }
                } else {
                    throw err;
                }
            }
        }
        throw lastErr;
    }

    // ── Helper Methods (Task 17) ───────────────

    /**
     * Return the first document matching where, or null if no match.
     * @param {string} table
     * @param {string} [where='']
     * @returns {object|null}
     */
    findOne(table, where) {
        const w = where || '';
        const sql = w
            ? ('SELECT * FROM ' + table + ' WHERE ' + w + ' LIMIT 1')
            : ('SELECT * FROM ' + table + ' LIMIT 1');
        const rows = this.query(sql);
        return rows.length > 0 ? rows[0] : null;
    }

    /**
     * Return all documents matching where.
     * @param {string} table
     * @param {string} [where='']
     * @param {string} [orderBy='']
     * @param {number} [limit=0]
     * @returns {object[]}
     */
    findAll(table, where, orderBy, limit) {
        const w = where || '';
        const ob = orderBy || '';
        const lim = limit || 0;
        let sql = 'SELECT * FROM ' + table;
        if (w) sql += ' WHERE ' + w;
        if (ob) sql += ' ORDER BY ' + ob;
        if (lim > 0) sql += ' LIMIT ' + lim;
        return this.query(sql);
    }

    /**
     * Update all documents matching where.
     * @param {string} table
     * @param {string} where
     * @param {object} updates
     * @returns {number}
     */
    updateMany(table, where, updates) {
        const setClauses = Object.keys(updates)
            .map(k => k + ' = ' + JSON.stringify(updates[k]))
            .join(', ');
        const sql = 'UPDATE ' + table + ' SET ' + setClauses + ' WHERE ' + where;
        const result = this.queryFull(sql);
        return Number(result.rows_affected || 0);
    }

    /**
     * Delete all documents matching where.
     * @param {string} table
     * @param {string} where
     * @returns {number}
     */
    deleteMany(table, where) {
        const sql = 'DELETE FROM ' + table + ' WHERE ' + where;
        const result = this.queryFull(sql);
        return Number(result.rows_affected || 0);
    }

    /**
     * Count documents matching where.
     * @param {string} table
     * @param {string} [where='']
     * @returns {number}
     */
    countWhere(table, where) {
        const w = where || '';
        const sql = w
            ? ('SELECT COUNT(*) FROM ' + table + ' WHERE ' + w)
            : ('SELECT COUNT(*) FROM ' + table);
        const rows = this.query(sql);
        if (!rows || rows.length === 0) return 0;
        const row = rows[0];
        for (const key of ['COUNT(*)', 'count(*)', 'count', 'COUNT']) {
            if (key in row) return Number(row[key]);
        }
        const vals = Object.values(row);
        return vals.length > 0 ? Number(vals[0]) : 0;
    }

    /**
     * Check whether a document with the given id exists.
     * @param {string} table
     * @param {string} id
     * @returns {boolean}
     */
    exists(table, id) {
        return this.get(table, id) !== null;
    }

    _ensureOpen() {
        if (!this._handle) {
            throw new OverDriveError('Database is closed');
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
            execSync('icacls "' + filePath + '" /inheritance:r /grant:r "%USERNAME%:F"', { stdio: 'pipe' });
        } catch (e) {
            process.emitWarning('[overdrive] Could not harden file permissions on \'' + filePath + '\': ' + e.message);
        }
    } else {
        try {
            fs.chmodSync(filePath, 0o600);
        } catch (e) {
            process.emitWarning('[overdrive] Could not chmod 600 \'' + filePath + '\': ' + e.message);
        }
    }
}

// Patch _setSecurePermissions into constructor (backward compat)
const _OriginalOverDrive = OverDrive;
class OverDriveSecure extends _OriginalOverDrive {
    constructor(dbPath) {
        super(dbPath);
        _setSecurePermissions(dbPath);
    }
}
Object.defineProperty(OverDriveSecure, 'name', { value: 'OverDrive' });

// ── Async-safe Shared Wrapper ─────────────────

/**
 * SharedOverDrive — async-safe OverDrive wrapper using a promise-based mutex.
 */
class SharedOverDrive {
    constructor(dbPath) {
        this._db = new OverDrive(dbPath);
        this._queue = Promise.resolve();
    }

    static openEncrypted(dbPath, keyEnvVar) {
        const instance = new SharedOverDrive.__proto__(dbPath);
        instance._db = OverDrive.openEncrypted(dbPath, keyEnvVar || 'ODB_KEY');
        instance._queue = Promise.resolve();
        return instance;
    }

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

module.exports = {
    OverDrive,
    SharedOverDrive,
    OverDriveError,
    AuthenticationError,
    TableError,
    QueryError,
    TransactionError,
    OverDriveIOError,
    FFIError,
};
