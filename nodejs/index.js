'use strict';
/**
 * OverDrive-DB Node.js SDK v2.3.3
 * Full engine API: Disk, RAM, Vector, Graph, Time-Series, Streaming
 */

const { getLib } = require('./lib');

// ── FFI bindings ─────────────────────────────────────────────────────────────
let _ffi = null;
function ffi() {
    if (_ffi) return _ffi;
    const lib = getLib();
    const k = require('koffi');

    _ffi = {
        // Core
        open:               lib.func('void * overdrive_open(const char *path)'),
        open_with_engine:   lib.func('void * overdrive_open_with_engine(const char *path, const char *engine, const char *opts)'),
        open_with_password: lib.func('void * overdrive_open_with_password(const char *path, const char *password)'),
        close:              lib.func('void overdrive_close(void *handle)'),
        sync:               lib.func('void overdrive_sync(void *handle)'),
        // version/last_error: const char* — Koffi can auto-convert, no free needed
        version:            lib.func('const char * overdrive_version()'),
        free_string:        lib.func('void overdrive_free_string(void *ptr)'),
        last_error:         lib.func('const char * overdrive_last_error()'),
        last_error_ex:      lib.func('const char * overdrive_last_error_ex(void *handle)'),
        // Tables
        create_table:       lib.func('int overdrive_create_table(void *handle, const char *name)'),
        drop_table:         lib.func('int overdrive_drop_table(void *handle, const char *name)'),
        list_tables:        lib.func('void * overdrive_list_tables(void *handle)'),
        table_exists:       lib.func('int overdrive_table_exists(void *handle, const char *name)'),
        // CRUD — void* keeps raw pointer so we can free it ourselves
        insert:             lib.func('void * overdrive_insert(void *handle, const char *table, const char *json)'),
        get:                lib.func('void * overdrive_get(void *handle, const char *table, const char *id)'),
        update:             lib.func('int overdrive_update(void *handle, const char *table, const char *id, const char *json)'),
        delete:             lib.func('int overdrive_delete(void *handle, const char *table, const char *id)'),
        count:              lib.func('int overdrive_count(void *handle, const char *table)'),
        get_history:        lib.func('void * overdrive_get_history(void *handle, const char *table, const char *id)'),
        // Query / Search
        query:              lib.func('void * overdrive_query(void *handle, const char *sql)'),
        query_safe:         lib.func('void * overdrive_query_safe(void *handle, const char *sql, const char *params)'),
        search:             lib.func('void * overdrive_search(void *handle, const char *table, const char *text)'),
        // Transactions
        begin_txn:          lib.func('uint64_t overdrive_begin_transaction(void *handle, int iso)'),
        commit_txn:         lib.func('int overdrive_commit_transaction(void *handle, uint64_t txn_id)'),
        abort_txn:          lib.func('int overdrive_abort_transaction(void *handle, uint64_t txn_id)'),
        // Integrity / Backup
        verify_integrity:   lib.func('void * overdrive_verify_integrity(void *handle)'),
        backup:             lib.func('void * overdrive_backup(void *handle, const char *dest)'),
        cleanup_wal:        lib.func('int overdrive_cleanup_wal(void *handle)'),
        // Engine info
        get_engine_type:    lib.func('void * overdrive_get_engine_type(void *handle)'),
        memory_usage:       lib.func('void * overdrive_memory_usage(void *handle)'),
        set_auto_create:    lib.func('void overdrive_set_auto_create_tables(void *handle, int enabled)'),
        // RAM engine
        create_ram_db:      lib.func('void * overdrive_create_ram_db()'),
        create_ram_table:   lib.func('int overdrive_create_ram_table(void *handle, const char *name, const char *schema)'),
        snapshot:           lib.func('int overdrive_snapshot(void *handle, const char *dest)'),
        restore:            lib.func('int overdrive_restore(void *handle, const char *src)'),
        // Time-Series engine
        create_timeseries:    lib.func('int overdrive_create_timeseries(void *handle, const char *name, uint64_t ttl_seconds)'),
        insert_measurement:   lib.func('int overdrive_insert_measurement(void *handle, const char *series, const char *measurement_json)'),
        query_timeseries:     lib.func('void * overdrive_query_timeseries(void *handle, const char *series, int64_t start_ts, int64_t end_ts)'),
        aggregate_timeseries: lib.func('void * overdrive_aggregate_timeseries(void *handle, const char *series, int64_t start_ts, int64_t end_ts, int64_t window_sec, const char *aggregation)'),
        drop_timeseries:      lib.func('int overdrive_drop_timeseries(void *handle, const char *name)'),
        list_timeseries:      lib.func('void * overdrive_list_timeseries(void *handle)'),
        // Vector engine
        create_vector_index:  lib.func('int overdrive_create_vector_index(void *handle, const char *table, const char *field, uint32_t dimensions, const char *metric)'),
        insert_vector:        lib.func('void * overdrive_insert_vector(void *handle, const char *table, const char *json_doc, const char *embedding_json)'),
        vector_search:        lib.func('void * overdrive_vector_search(void *handle, const char *table, const char *query_vec_json, uint32_t limit, const char *metric)'),
        drop_vector_index:    lib.func('int overdrive_drop_vector_index(void *handle, const char *table)'),
        list_vector_indexes:  lib.func('void * overdrive_list_vector_indexes(void *handle)'),
        // Graph engine
        create_node_type:     lib.func('int overdrive_create_node_type(void *handle, const char *type_name)'),
        create_edge_type:     lib.func('int overdrive_create_edge_type(void *handle, const char *type_name)'),
        create_node:          lib.func('void * overdrive_create_node(void *handle, const char *type_name, const char *props_json)'),
        create_edge:          lib.func('void * overdrive_create_edge(void *handle, const char *edge_type, const char *from_id, const char *to_id, const char *props_json)'),
        graph_traverse:       lib.func('void * overdrive_graph_traverse(void *handle, const char *match_query)'),
        shortest_path:        lib.func('void * overdrive_shortest_path(void *handle, const char *from_id, const char *to_id)'),
        delete_node:          lib.func('int overdrive_delete_node(void *handle, const char *node_id)'),
        list_nodes:           lib.func('void * overdrive_list_nodes(void *handle, const char *type_name)'),
        // Streaming engine
        create_topic:         lib.func('int overdrive_create_topic(void *handle, const char *topic_name, uint32_t partitions, uint64_t retention_seconds)'),
        publish:              lib.func('void * overdrive_publish(void *handle, const char *topic_name, const char *message_json)'),
        subscribe:            lib.func('void * overdrive_subscribe(void *handle, const char *topic_name, const char *consumer_group, const char *offset_mode)'),
        poll:                 lib.func('void * overdrive_poll(void *handle, uint64_t subscription_id, uint32_t max_messages, uint32_t timeout_ms)'),
        commit_offset:        lib.func('int overdrive_commit_offset(void *handle, const char *topic_name, const char *consumer_group, uint64_t offset)'),
        unsubscribe:          lib.func('int overdrive_unsubscribe(void *handle, uint64_t subscription_id)'),
        drop_topic:           lib.func('int overdrive_drop_topic(void *handle, const char *topic)'),
        list_topics:          lib.func('void * overdrive_list_topics(void *handle)'),
    };
    return _ffi;
}

// ── Helpers ──────────────────────────────────────────────────────────────────
function _err(h, op) {
    // set_error() in Rust writes to a thread-local; last_error_ex reads the
    // handle-local copy (which may be empty).  Fall back to thread-local so we
    // always surface the real error message instead of "unknown error".
    let msg = h ? ffi().last_error_ex(h) : null;
    if (!msg) msg = ffi().last_error();
    return new Error(`[overdrive-db] ${op} failed: ${msg || 'unknown error'}`);
}

/**
 * SECURITY: prototype-pollution-safe JSON parse.
 * Rejects payloads that set __proto__, constructor, or prototype keys.
 */
function _safeJson(str) {
    if (!str) return null;
    const obj = JSON.parse(str);
    if (obj && typeof obj === 'object' && !Array.isArray(obj)) {
        if (Object.prototype.hasOwnProperty.call(obj, '__proto__') ||
            Object.prototype.hasOwnProperty.call(obj, 'prototype')) {
            throw new Error('[overdrive-db] Rejected: dangerous prototype key in response');
        }
    }
    return obj;
}

/**
 * MEMORY: read a Rust-heap pointer into a JS string then free the original pointer.
 *
 * WHY void* not char*:
 *   When Koffi sees `char *` as a return type it auto-converts to a JS string
 *   and DISCARDS the original C pointer.  _readAndFree then passes the JS
 *   string to free_string — Koffi wraps it in a fresh malloc'd buffer, and
 *   Rust's CString::from_raw tries to free that malloc'd memory with its own
 *   allocator → SIGSEGV.
 *
 *   By declaring `void *`, Koffi keeps the raw pointer. We decode it with
 *   koffi.decode(), then pass the REAL Rust-heap pointer to free_string.
 */
const koffi = require('koffi');

function _readAndFree(ptr, op, handle) {
    if (!ptr) throw _err(handle, op);
    // Decode the Rust-heap bytes to a JS string without losing the pointer
    const str = koffi.decode(ptr, 'char', -1);  // -1 = null-terminated
    // Free the REAL Rust-allocated pointer — not a JS string copy
    try { ffi().free_string(ptr); } catch (_) { /* best effort */ }
    return str;
}

function _readAndFreeNullable(ptr) {
    if (!ptr) return null;
    const str = koffi.decode(ptr, 'char', -1);
    try { ffi().free_string(ptr); } catch (_) { /* best effort */ }
    return str;
}

// ── Isolation levels ─────────────────────────────────────────────────────────
const IsolationLevel = {
    ReadUncommitted: 0,
    ReadCommitted:   1,
    RepeatableRead:  2,
    Serializable:    3,
};

// ── Main class ───────────────────────────────────────────────────────────────
class OverdriveDb {

    constructor(handle) {
        this._handle = handle;
        this._closed = false;
    }

    _assertOpen() {
        if (this._closed || !this._handle)
            throw new Error('[overdrive-db] Database handle is already closed');
    }

    // ── Static ───────────────────────────────────────────────────────────────

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
        if (!handle) throw _err(null, 'open');
        return new OverdriveDb(handle);
    }

    static version() { return ffi().version(); }

    // ── Lifecycle ─────────────────────────────────────────────────────────────

    close() {
        if (this._handle && !this._closed) {
            ffi().close(this._handle);
            this._handle = null;
            this._closed = true;
        }
    }
    sync()  { this._assertOpen(); ffi().sync(this._handle); }
    getEngineType() {
        this._assertOpen();
        return _readAndFreeNullable(ffi().get_engine_type(this._handle)) || 'Disk';
    }
    memoryUsage() {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().memory_usage(this._handle));
        return s ? JSON.parse(s) : {};
    }
    setAutoCreateTables(enabled) { this._assertOpen(); ffi().set_auto_create(this._handle, enabled ? 1 : 0); }

    // ── Tables ────────────────────────────────────────────────────────────────

    createTable(name) {
        this._assertOpen();
        if (ffi().create_table(this._handle, name) !== 0) throw _err(this._handle, 'createTable');
    }
    dropTable(name) {
        this._assertOpen();
        if (ffi().drop_table(this._handle, name) !== 0) throw _err(this._handle, 'dropTable');
    }
    listTables() {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().list_tables(this._handle));
        return s ? JSON.parse(s) : [];
    }
    tableExists(name) { this._assertOpen(); return ffi().table_exists(this._handle, name) === 1; }

    // ── CRUD ──────────────────────────────────────────────────────────────────

    insert(table, doc) {
        this._assertOpen();
        const id = _readAndFree(ffi().insert(this._handle, table, JSON.stringify(doc)), 'insert', this._handle);
        return id;
    }
    insertMany(table, docs) { return docs.map(d => this.insert(table, d)); }

    get(table, id) {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().get(this._handle, table, id));
        return s ? _safeJson(s) : null;
    }
    update(table, id, patch) {
        this._assertOpen();
        const r = ffi().update(this._handle, table, id, JSON.stringify(patch));
        if (r === -1) throw _err(this._handle, 'update');
        return r === 1;
    }
    delete(table, id) {
        this._assertOpen();
        const r = ffi().delete(this._handle, table, id);
        if (r === -1) throw _err(this._handle, 'delete');
        return r === 1;
    }
    count(table) {
        this._assertOpen();
        const n = ffi().count(this._handle, table);
        if (n < 0) throw _err(this._handle, 'count');
        return n;
    }
    getHistory(table, id) {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().get_history(this._handle, table, id));
        return s ? JSON.parse(s) : [];
    }

    // ── Query / Search ────────────────────────────────────────────────────────

    query(sql) {
        this._assertOpen();
        const s = _readAndFree(ffi().query(this._handle, sql), 'query', this._handle);
        const res = JSON.parse(s);
        if (res.rows !== undefined) return res.rows;
        if (res.result !== undefined) return [{ result: res.result }];
        return Array.isArray(res) ? res : [res];
    }
    querySafe(sql, params = []) {
        this._assertOpen();
        const s = _readAndFree(ffi().query_safe(this._handle, sql, JSON.stringify(params)), 'querySafe', this._handle);
        const res = JSON.parse(s);
        if (res.rows !== undefined) return res.rows;
        return Array.isArray(res) ? res : [res];
    }
    search(table, text) {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().search(this._handle, table, text));
        return s ? JSON.parse(s) : [];
    }

    // ── Transactions ──────────────────────────────────────────────────────────

    beginTransaction(isolationLevel = IsolationLevel.ReadCommitted) {
        const id = ffi().begin_txn(this._handle, isolationLevel);
        if (!id) throw _err(this._handle, 'beginTransaction');
        return { id };
    }
    commitTransaction(txn) {
        if (ffi().commit_txn(this._handle, txn.id) !== 0) throw _err(this._handle, 'commit');
    }
    abortTransaction(txn) { ffi().abort_txn(this._handle, txn.id); }
    transaction(fn, isolationLevel = IsolationLevel.ReadCommitted) {
        const txn = this.beginTransaction(isolationLevel);
        try { const r = fn(this); this.commitTransaction(txn); return r; }
        catch (e) { this.abortTransaction(txn); throw e; }
    }

    // ── Integrity / Backup ────────────────────────────────────────────────────

    verifyIntegrity() {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().verify_integrity(this._handle));
        return s ? JSON.parse(s) : null;
    }
    backup(destPath) {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().backup(this._handle, destPath));
        return s ? JSON.parse(s) : null;
    }
    cleanupWal() { this._assertOpen(); return ffi().cleanup_wal(this._handle); }

    // ══════════════════════════════════════════════════════════════════════════
    // TIME-SERIES ENGINE
    // ══════════════════════════════════════════════════════════════════════════

    /**
     * Create a time-series collection.
     * @param {string} name
     * @param {number} [ttlSeconds=0] - Retention window in seconds (0 = keep forever)
     */
    createTimeseries(name, ttlSeconds = 0) {
        this._assertOpen();
        const r = ffi().create_timeseries(this._handle, name, ttlSeconds);
        if (r !== 0) throw _err(this._handle, 'createTimeseries');
    }

    /**
     * Insert a measurement. The JSON object must have at least one numeric field.
     * Include `timestamp` (Unix seconds) or `ts` to set an explicit time.
     * @param {string} series
     * @param {object} data  e.g. { value: 42.5, sensor: 'A', timestamp: 1700000000 }
     */
    insertMeasurement(series, data) {
        this._assertOpen();
        const r = ffi().insert_measurement(this._handle, series, JSON.stringify(data));
        if (r !== 0) throw _err(this._handle, 'insertMeasurement');
    }

    /**
     * Query raw measurements in a Unix-second time range.
     * @param {string} series
     * @param {number} startTs - Unix timestamp in seconds
     * @param {number} endTs   - Unix timestamp in seconds
     * @returns {object[]}
     */
    queryTimeseries(series, startTs, endTs) {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().query_timeseries(this._handle, series, startTs, endTs));
        return s ? JSON.parse(s) : [];
    }

    /**
     * Aggregate measurements in a time window.
     * @param {string} series
     * @param {number} startTs    - Unix timestamp seconds
     * @param {number} endTs      - Unix timestamp seconds
     * @param {number} windowSec  - Bucket size in seconds (e.g. 60 for 1-minute buckets)
     * @param {'avg'|'sum'|'min'|'max'|'count'} func
     * @returns {object[]}
     */
    aggregateTimeseries(series, startTs, endTs, windowSec, func = 'avg') {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().aggregate_timeseries(this._handle, series, startTs, endTs, windowSec, func));
        return s ? JSON.parse(s) : [];
    }

    dropTimeseries(name) {
        this._assertOpen();
        if (ffi().drop_timeseries(this._handle, name) !== 0) throw _err(this._handle, 'dropTimeseries');
    }
    listTimeseries() {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().list_timeseries(this._handle));
        return s ? JSON.parse(s) : [];
    }

    // ══════════════════════════════════════════════════════════════════════════
    // VECTOR ENGINE
    // ══════════════════════════════════════════════════════════════════════════

    /**
     * Create a vector index (collection).
     * @param {string} name       - Index / collection name
     * @param {number} dims       - Embedding dimensions (e.g. 1536 for OpenAI ada-002)
     * @param {string} [metric]   - 'cosine' (default) | 'euclidean' | 'dot'
     * @param {string} [field]    - Reserved field name (pass '' for default)
     */
    createVectorIndex(name, dims, metric = 'cosine', field = '') {
        const r = ffi().create_vector_index(this._handle, name, field, dims, metric);
        if (r !== 0) throw _err(this._handle, 'createVectorIndex');
    }

    /**
     * Insert a vector embedding.
     * @param {string}   collection  - Collection / index name
     * @param {number[]} vec         - Embedding array e.g. [0.1, 0.2, ...]
     * @param {object}   [meta]      - Metadata stored alongside the vector.
     *                                 Include `_id` to use a specific id.
     * @returns {string} The generated or provided document id.
     */
    insertVector(collection, vec, meta = {}) {
        this._assertOpen();
        const id = _readAndFree(
            ffi().insert_vector(this._handle, collection, JSON.stringify(meta), JSON.stringify(vec)),
            'insertVector', this._handle
        );
        return id;
    }

    vectorSearch(collection, vec, topK = 10, metric = 'cosine') {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().vector_search(this._handle, collection, JSON.stringify(vec), topK, metric));
        return s ? JSON.parse(s) : [];
    }

    dropVectorIndex(name) {
        this._assertOpen();
        if (ffi().drop_vector_index(this._handle, name) !== 0) throw _err(this._handle, 'dropVectorIndex');
    }
    listVectorIndexes() {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().list_vector_indexes(this._handle));
        return s ? JSON.parse(s) : [];
    }

    // ══════════════════════════════════════════════════════════════════════════
    // GRAPH ENGINE
    // ══════════════════════════════════════════════════════════════════════════

    /**
     * Define a node type. No schema needed — OverDrive-DB is schemaless.
     * @param {string} typeName  e.g. 'Person', 'Product'
     */
    createNodeType(typeName) {
        const r = ffi().create_node_type(this._handle, typeName);
        if (r !== 0) throw _err(this._handle, 'createNodeType');
    }

    /**
     * Define an edge type (relationship).
     * @param {string} typeName  e.g. 'KNOWS', 'BOUGHT'
     */
    createEdgeType(typeName) {
        const r = ffi().create_edge_type(this._handle, typeName);
        if (r !== 0) throw _err(this._handle, 'createEdgeType');
    }

    /**
     * Create a graph node. Returns the new node id.
     * @param {string} typeName
     * @param {object} [props]
     * @returns {string}
     */
    createNode(typeName, props = {}) {
        this._assertOpen();
        const id = _readAndFree(ffi().create_node(this._handle, typeName, JSON.stringify(props)), 'createNode', this._handle);
        return id;
    }

    createEdge(typeName, fromId, toId, props = {}) {
        this._assertOpen();
        // Rust returns void* (edge ID string) — must _readAndFree, not check === 0
        return _readAndFree(ffi().create_edge(this._handle, typeName, fromId, toId, JSON.stringify(props)), 'createEdge', this._handle);
    }

    graphTraverse(query) {
        this._assertOpen();
        // Rust takes a single match_query string (Cypher-like), not (startId, direction, maxDepth)
        const s = _readAndFreeNullable(ffi().graph_traverse(this._handle, query));
        return s ? JSON.parse(s) : [];
    }


    shortestPath(fromId, toId) {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().shortest_path(this._handle, fromId, toId));
        return s ? JSON.parse(s) : [];
    }

    deleteNode(nodeId) {
        this._assertOpen();
        if (ffi().delete_node(this._handle, nodeId) !== 0) throw _err(this._handle, 'deleteNode');
    }
    listNodes(typeName = '') {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().list_nodes(this._handle, typeName));
        return s ? JSON.parse(s) : [];
    }

    // ══════════════════════════════════════════════════════════════════════════
    // STREAMING ENGINE
    // ══════════════════════════════════════════════════════════════════════════

    /**
     * Create a streaming topic.
     * @param {string} topic
     * @param {number} [partitions=1]
     * @param {number} [retentionSeconds=0] - 0 = keep forever
     */
    createTopic(topic, partitions = 1, retentionSeconds = 0) {
        this._assertOpen();
        const r = ffi().create_topic(this._handle, topic, partitions, retentionSeconds);
        if (r !== 0) throw _err(this._handle, 'createTopic');
    }

    /**
     * Publish a message. Returns the assigned offset number.
     * @param {string} topic
     * @param {string|object} message  - Will be JSON-stringified if object
     * @returns {number} offset
     */
    publish(topic, message) {
        this._assertOpen();
        const msg = typeof message === 'string' ? message : JSON.stringify(message);
        const s = _readAndFree(ffi().publish(this._handle, topic, msg), 'publish', this._handle);
        const obj = JSON.parse(s);
        return obj.offset;
    }

    /**
     * Subscribe to a topic. Returns subscription id (number).
     * @param {string} topic
     * @param {string} [consumerGroup='']
     * @param {'latest'|'earliest'} [offsetMode='latest']
     * @returns {number} subscriptionId
     */
    subscribe(topic, consumerGroup = '', offsetMode = 'latest') {
        this._assertOpen();
        const s = _readAndFree(ffi().subscribe(this._handle, topic, consumerGroup, offsetMode), 'subscribe', this._handle);
        const obj = JSON.parse(s);
        return obj.subscription_id;
    }

    /**
     * Poll messages from a subscription.
     * @param {number} subId     - From subscribe()
     * @param {number} [maxMsgs=100]
     * @param {number} [timeoutMs=0]
     * @returns {{ offset, timestamp_ms, payload }[]}
     */
    poll(subId, maxMsgs = 100, timeoutMs = 0) {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().poll(this._handle, subId, maxMsgs, timeoutMs));
        return s ? JSON.parse(s) : [];
    }

    /**
     * Commit a consumer group offset for durable consumption.
     * @param {string} topic
     * @param {string} consumerGroup
     * @param {number} offset
     */
    commitOffset(topic, consumerGroup, offset) {
        this._assertOpen();
        const r = ffi().commit_offset(this._handle, topic, consumerGroup, offset);
        if (r !== 0) throw _err(this._handle, 'commitOffset');
    }

    unsubscribe(subId) {
        this._assertOpen();
        ffi().unsubscribe(this._handle, subId);
    }
    dropTopic(topic) {
        this._assertOpen();
        if (ffi().drop_topic(this._handle, topic) !== 0) throw _err(this._handle, 'dropTopic');
    }
    listTopics() {
        this._assertOpen();
        const s = _readAndFreeNullable(ffi().list_topics(this._handle));
        return s ? JSON.parse(s) : [];
    }
}

module.exports = { OverdriveDb, OverDrive: OverdriveDb, IsolationLevel };
