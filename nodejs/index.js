'use strict';
/**
 * OverDrive-DB Node.js SDK v1.3.1
 * Class:    OverdriveDb
 * Instance: odb  (convention)
 *
 * Example:
 *   const { OverdriveDb } = require('overdrive-db');
 *   const odb = OverdriveDb.open('app.odb');
 *   const id  = odb.insert('users', { name: 'Alice' });
 *   const doc = odb.get('users', id);
 *   odb.close();
 */

const { getLib } = require('./lib');

// ── FFI type definitions ─────────────────────────────────────────────────────
let _ffi = null;
function ffi() {
    if (_ffi) return _ffi;
    const lib = getLib();
    const koffi = require('koffi');

    _ffi = {
        open:               lib.func('void * overdrive_open(const char *path)'),
        open_with_engine:   lib.func('void * overdrive_open_with_engine(const char *path, const char *engine, const char *opts)'),
        close:              lib.func('void overdrive_close(void *handle)'),
        sync:               lib.func('void overdrive_sync(void *handle)'),
        version:            lib.func('const char * overdrive_version()'),
        free_string:        lib.func('void overdrive_free_string(void *ptr)'),
        last_error:         lib.func('const char * overdrive_last_error()'),
        create_table:       lib.func('int overdrive_create_table(void *handle, const char *name)'),
        drop_table:         lib.func('int overdrive_drop_table(void *handle, const char *name)'),
        list_tables:        lib.func('char * overdrive_list_tables(void *handle)'),
        table_exists:       lib.func('int overdrive_table_exists(void *handle, const char *name)'),
        insert:             lib.func('char * overdrive_insert(void *handle, const char *table, const char *json)'),
        get:                lib.func('char * overdrive_get(void *handle, const char *table, const char *id)'),
        update:             lib.func('int overdrive_update(void *handle, const char *table, const char *id, const char *json)'),
        delete:             lib.func('int overdrive_delete(void *handle, const char *table, const char *id)'),
        count:              lib.func('int overdrive_count(void *handle, const char *table)'),
        query:              lib.func('char * overdrive_query(void *handle, const char *sql)'),
        search:             lib.func('char * overdrive_search(void *handle, const char *table, const char *text)'),
        begin_txn:          lib.func('uint64_t overdrive_begin_transaction(void *handle, int iso)'),
        commit_txn:         lib.func('int overdrive_commit_transaction(void *handle, uint64_t txn_id)'),
        abort_txn:          lib.func('int overdrive_abort_transaction(void *handle, uint64_t txn_id)'),
        verify_integrity:   lib.func('char * overdrive_verify_integrity(void *handle)'),
    };
    return _ffi;
}


// ── Isolation levels ─────────────────────────────────────────────────────────
const IsolationLevel = {
    ReadUncommitted: 0,
    ReadCommitted:   1,
    RepeatableRead:  2,
    Serializable:    3,
};

// ── OverdriveDb class ─────────────────────────────────────────────────────────
class OverdriveDb {

    constructor(handle) {
        this._handle = handle;
    }

    // ── Static ───────────────────────────────────────────────────────────

    /**
     * Open or create a database.
     * @param {string} path  - Path to .odb file
     * @param {object} [opts] - { password, engine, autoCreateTables }
     * @returns {OverdriveDb}
     */
    static open(path, opts = {}) {
        let handle;
        if (opts.engine || opts.password) {
            const engine = opts.engine || 'Disk';
            const options = JSON.stringify({
                password: opts.password || null,
                auto_create_tables: opts.autoCreateTables !== false,
            });
            handle = ffi().open_with_engine(path, engine, options);
        } else {
            handle = ffi().open(path);
        }
        if (!handle) throw new Error(`[overdrive-db] open failed: ${ffi().last_error()}`);
        return new OverdriveDb(handle);
    }

    /** Return native library version string. */
    static version() {
        return ffi().version();
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────

    close()  { ffi().close(this._handle); this._handle = null; }
    sync()   { ffi().sync(this._handle); }

    // ── Tables ────────────────────────────────────────────────────────────

    createTable(name)  {
        if (ffi().create_table(this._handle, name) !== 0)
            throw new Error(`[overdrive-db] createTable failed: ${ffi().last_error()}`);
    }

    dropTable(name) {
        if (ffi().drop_table(this._handle, name) !== 0)
            throw new Error(`[overdrive-db] dropTable failed: ${ffi().last_error()}`);
    }

    listTables() {
        const s = ffi().list_tables(this._handle);
        return s ? JSON.parse(s) : [];
    }

    tableExists(name) {
        return ffi().table_exists(this._handle, name) === 1;
    }

    // ── CRUD ──────────────────────────────────────────────────────────────

    /** Insert a document. Returns the generated _id string. */
    insert(table, doc) {
        const id = ffi().insert(this._handle, table, JSON.stringify(doc));
        if (!id) throw new Error(`[overdrive-db] insert failed: ${ffi().last_error()}`);
        return id;
    }

    /** Insert multiple documents. Returns array of _ids. */
    insertMany(table, docs) {
        return docs.map(doc => this.insert(table, doc));
    }

    /** Get a document by _id. Returns object or null. */
    get(table, id) {
        const s = ffi().get(this._handle, table, id);
        return s ? JSON.parse(s) : null;
    }

    /** Update a document by _id. Returns true if updated. */
    update(table, id, patch) {
        const r = ffi().update(this._handle, table, id, JSON.stringify(patch));
        if (r === -1) throw new Error(`[overdrive-db] update failed: ${ffi().last_error()}`);
        return r === 1;
    }

    /** Delete a document by _id. Returns true if deleted. */
    delete(table, id) {
        const r = ffi().delete(this._handle, table, id);
        if (r === -1) throw new Error(`[overdrive-db] delete failed: ${ffi().last_error()}`);
        return r === 1;
    }

    /** Count documents in a table. */
    count(table) {
        const n = ffi().count(this._handle, table);
        if (n < 0) throw new Error(`[overdrive-db] count failed: ${ffi().last_error()}`);
        return n;
    }

    // ── Query ─────────────────────────────────────────────────────────────

    /**
     * Execute a SQL query. Returns an array of result rows.
     * Supports: SELECT, WHERE, ORDER BY, LIMIT
     */
    query(sql) {
        const s = ffi().query(this._handle, sql);
        if (!s) throw new Error(`[overdrive-db] query failed: ${ffi().last_error()}`);
        const res = JSON.parse(s);
        if (res.rows !== undefined) return res.rows;
        if (res.result !== undefined) return [{ result: res.result }];
        return [res];
    }

    /** Full-text search across a table. */
    search(table, text) {
        const s = ffi().search(this._handle, table, text);
        return s ? JSON.parse(s) : [];
    }

    // ── Transactions ──────────────────────────────────────────────────────

    beginTransaction(isolationLevel = IsolationLevel.ReadCommitted) {
        const id = ffi().begin_txn(this._handle, isolationLevel);
        if (!id) throw new Error(`[overdrive-db] beginTransaction failed: ${ffi().last_error()}`);
        return { id };
    }

    commitTransaction(txn) {
        if (ffi().commit_txn(this._handle, txn.id) !== 0)
            throw new Error(`[overdrive-db] commit failed: ${ffi().last_error()}`);
    }

    abortTransaction(txn) {
        ffi().abort_txn(this._handle, txn.id);
    }

    /**
     * Run fn(odb) inside a transaction.
     * Auto-commits on success, auto-aborts on error.
     */
    transaction(fn, isolationLevel = IsolationLevel.ReadCommitted) {
        const txn = this.beginTransaction(isolationLevel);
        try {
            const result = fn(this);
            this.commitTransaction(txn);
            return result;
        } catch (e) {
            this.abortTransaction(txn);
            throw e;
        }
    }

    // ── Integrity ─────────────────────────────────────────────────────────

    verifyIntegrity() {
        const s = ffi().verify_integrity(this._handle);
        return s ? JSON.parse(s) : null;
    }
}

module.exports = { OverdriveDb, IsolationLevel };
