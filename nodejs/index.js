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
        version:            lib.func('const char * overdrive_version()'),
        free_string:        lib.func('void overdrive_free_string(void *ptr)'),
        last_error:         lib.func('const char * overdrive_last_error()'),
        last_error_ex:      lib.func('const char * overdrive_last_error_ex(void *handle)'),
        // Tables
        create_table:       lib.func('int overdrive_create_table(void *handle, const char *name)'),
        drop_table:         lib.func('int overdrive_drop_table(void *handle, const char *name)'),
        list_tables:        lib.func('char * overdrive_list_tables(void *handle)'),
        table_exists:       lib.func('int overdrive_table_exists(void *handle, const char *name)'),
        // CRUD
        insert:             lib.func('char * overdrive_insert(void *handle, const char *table, const char *json)'),
        get:                lib.func('char * overdrive_get(void *handle, const char *table, const char *id)'),
        update:             lib.func('int overdrive_update(void *handle, const char *table, const char *id, const char *json)'),
        delete:             lib.func('int overdrive_delete(void *handle, const char *table, const char *id)'),
        count:              lib.func('int overdrive_count(void *handle, const char *table)'),
        get_history:        lib.func('char * overdrive_get_history(void *handle, const char *table, const char *id)'),
        // Query / Search
        query:              lib.func('char * overdrive_query(void *handle, const char *sql)'),
        query_safe:         lib.func('char * overdrive_query_safe(void *handle, const char *sql, const char *params)'),
        search:             lib.func('char * overdrive_search(void *handle, const char *table, const char *text)'),
        // Transactions
        begin_txn:          lib.func('uint64_t overdrive_begin_transaction(void *handle, int iso)'),
        commit_txn:         lib.func('int overdrive_commit_transaction(void *handle, uint64_t txn_id)'),
        abort_txn:          lib.func('int overdrive_abort_transaction(void *handle, uint64_t txn_id)'),
        // Integrity / Backup
        verify_integrity:   lib.func('char * overdrive_verify_integrity(void *handle)'),
        backup:             lib.func('char * overdrive_backup(void *handle, const char *dest)'),
        cleanup_wal:        lib.func('int overdrive_cleanup_wal(void *handle)'),
        // Engine info
        get_engine_type:    lib.func('char * overdrive_get_engine_type(void *handle)'),
        memory_usage:       lib.func('char * overdrive_memory_usage(void *handle)'),
        set_auto_create:    lib.func('void overdrive_set_auto_create_tables(void *handle, int enabled)'),
        // RAM engine
        create_ram_db:      lib.func('void * overdrive_create_ram_db()'),
        create_ram_table:   lib.func('int overdrive_create_ram_table(void *handle, const char *name, const char *schema)'),
        snapshot:           lib.func('int overdrive_snapshot(void *handle, const char *dest)'),
        restore:            lib.func('int overdrive_restore(void *handle, const char *src)'),
        // Time-Series engine
        create_timeseries:  lib.func('int overdrive_create_timeseries(void *handle, const char *name, const char *opts)'),
        insert_measurement: lib.func('int overdrive_insert_measurement(void *handle, const char *series, const char *json)'),
        query_timeseries:   lib.func('char * overdrive_query_timeseries(void *handle, const char *series, const char *from, const char *to, int limit)'),
        aggregate_timeseries: lib.func('char * overdrive_aggregate_timeseries(void *handle, const char *series, const char *func, const char *from, const char *to)'),
        drop_timeseries:    lib.func('int overdrive_drop_timeseries(void *handle, const char *name)'),
        list_timeseries:    lib.func('char * overdrive_list_timeseries(void *handle)'),
        // Vector engine
        create_vector_index: lib.func('int overdrive_create_vector_index(void *handle, const char *table, const char *field, uint32_t dimensions, const char *metric)'),
        insert_vector:       lib.func('char * overdrive_insert_vector(void *handle, const char *table, const char *json_doc, const char *embedding_json)'),
        vector_search:       lib.func('char * overdrive_vector_search(void *handle, const char *table, const char *query_vec_json, uint32_t limit, const char *metric)'),
        drop_vector_index:   lib.func('int overdrive_drop_vector_index(void *handle, const char *table)'),
        list_vector_indexes: lib.func('char * overdrive_list_vector_indexes(void *handle)'),
        // Graph engine
        create_node_type:    lib.func('int overdrive_create_node_type(void *handle, const char *type_name)'),
        create_edge_type:    lib.func('int overdrive_create_edge_type(void *handle, const char *type_name)'),
        create_node:         lib.func('char * overdrive_create_node(void *handle, const char *type_name, const char *props_json)'),
        create_edge:         lib.func('int overdrive_create_edge(void *handle, const char *edge_type, const char *from_id, const char *to_id, const char *props_json)'),
        graph_traverse:      lib.func('char * overdrive_graph_traverse(void *handle, const char *start_id, const char *direction, int max_depth)'),
        shortest_path:       lib.func('char * overdrive_shortest_path(void *handle, const char *from_id, const char *to_id)'),
        delete_node:         lib.func('int overdrive_delete_node(void *handle, const char *node_id)'),
        list_nodes:          lib.func('char * overdrive_list_nodes(void *handle, const char *type_name)'),
        // Streaming engine
        create_topic:       lib.func('int overdrive_create_topic(void *handle, const char *topic, const char *opts)'),
        publish:            lib.func('int overdrive_publish(void *handle, const char *topic, const char *message)'),
        subscribe:          lib.func('uint64_t overdrive_subscribe(void *handle, const char *topic, const char *group)'),
        poll:               lib.func('char * overdrive_poll(void *handle, uint64_t sub_id, int max_msgs)'),
        commit_offset:      lib.func('int overdrive_commit_offset(void *handle, uint64_t sub_id, uint64_t offset)'),
        unsubscribe:        lib.func('int overdrive_unsubscribe(void *handle, uint64_t sub_id)'),
        drop_topic:         lib.func('int overdrive_drop_topic(void *handle, const char *topic)'),
        list_topics:        lib.func('char * overdrive_list_topics(void *handle)'),
    };
    return _ffi;
}

// ── Helpers ──────────────────────────────────────────────────────────────────
function _err(h, op) {
    const msg = h ? ffi().last_error_ex(h) : ffi().last_error();
    return new Error(`[overdrive-db] ${op} failed: ${msg || 'unknown error'}`);
}
function _parsePtr(ptr, op, handle) {
    if (!ptr) throw _err(handle, op);
    try { return JSON.parse(ptr); } catch { return ptr; }
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

    constructor(handle) { this._handle = handle; }

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

    close() { if (this._handle) { ffi().close(this._handle); this._handle = null; } }
    sync()  { ffi().sync(this._handle); }
    getEngineType() { return ffi().get_engine_type(this._handle) || 'Disk'; }
    memoryUsage() { return _parsePtr(ffi().memory_usage(this._handle), 'memoryUsage', this._handle); }
    setAutoCreateTables(enabled) { ffi().set_auto_create(this._handle, enabled ? 1 : 0); }

    // ── Tables ────────────────────────────────────────────────────────────────

    createTable(name) {
        if (ffi().create_table(this._handle, name) !== 0) throw _err(this._handle, 'createTable');
    }
    dropTable(name) {
        if (ffi().drop_table(this._handle, name) !== 0) throw _err(this._handle, 'dropTable');
    }
    listTables() {
        const s = ffi().list_tables(this._handle);
        return s ? JSON.parse(s) : [];
    }
    tableExists(name) { return ffi().table_exists(this._handle, name) === 1; }

    // ── CRUD ──────────────────────────────────────────────────────────────────

    insert(table, doc) {
        const id = ffi().insert(this._handle, table, JSON.stringify(doc));
        if (!id) throw _err(this._handle, 'insert');
        return id;
    }
    insertMany(table, docs) { return docs.map(d => this.insert(table, d)); }

    get(table, id) {
        const s = ffi().get(this._handle, table, id);
        return s ? JSON.parse(s) : null;
    }
    update(table, id, patch) {
        const r = ffi().update(this._handle, table, id, JSON.stringify(patch));
        if (r === -1) throw _err(this._handle, 'update');
        return r === 1;
    }
    delete(table, id) {
        const r = ffi().delete(this._handle, table, id);
        if (r === -1) throw _err(this._handle, 'delete');
        return r === 1;
    }
    count(table) {
        const n = ffi().count(this._handle, table);
        if (n < 0) throw _err(this._handle, 'count');
        return n;
    }
    getHistory(table, id) {
        return _parsePtr(ffi().get_history(this._handle, table, id), 'getHistory', this._handle);
    }

    // ── Query / Search ────────────────────────────────────────────────────────

    query(sql) {
        const s = ffi().query(this._handle, sql);
        if (!s) throw _err(this._handle, 'query');
        const res = JSON.parse(s);
        if (res.rows !== undefined) return res.rows;
        if (res.result !== undefined) return [{ result: res.result }];
        return Array.isArray(res) ? res : [res];
    }
    querySafe(sql, params = []) {
        const s = ffi().query_safe(this._handle, sql, JSON.stringify(params));
        if (!s) throw _err(this._handle, 'querySafe');
        const res = JSON.parse(s);
        if (res.rows !== undefined) return res.rows;
        return Array.isArray(res) ? res : [res];
    }
    search(table, text) {
        const s = ffi().search(this._handle, table, text);
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
        const s = ffi().verify_integrity(this._handle);
        return s ? JSON.parse(s) : null;
    }
    backup(destPath) {
        return _parsePtr(ffi().backup(this._handle, destPath), 'backup', this._handle);
    }
    cleanupWal() { return ffi().cleanup_wal(this._handle); }

    // ══════════════════════════════════════════════════════════════════════════
    // TIME-SERIES ENGINE
    // ══════════════════════════════════════════════════════════════════════════

    /**
     * Create a time-series collection.
     * @param {string} name
     * @param {object} [opts] - { retention_secs, granularity }
     */
    createTimeseries(name, opts = {}) {
        const r = ffi().create_timeseries(this._handle, name, JSON.stringify(opts));
        if (r !== 0) throw _err(this._handle, 'createTimeseries');
    }

    /**
     * Insert a measurement into a time-series.
     * @param {string} series - Series name
     * @param {object} data   - { timestamp?: ISO string, value, tags? }
     */
    insertMeasurement(series, data) {
        const payload = { timestamp: new Date().toISOString(), ...data };
        const r = ffi().insert_measurement(this._handle, series, JSON.stringify(payload));
        if (r !== 0) throw _err(this._handle, 'insertMeasurement');
    }

    /**
     * Query a time-series range.
     * @param {string} series
     * @param {string} from  - ISO timestamp or null
     * @param {string} to    - ISO timestamp or null
     * @param {number} limit
     * @returns {object[]}
     */
    queryTimeseries(series, from = null, to = null, limit = 1000) {
        const s = ffi().query_timeseries(this._handle, series, from || '', to || '', limit);
        return s ? JSON.parse(s) : [];
    }

    /**
     * Aggregate a time-series (count/sum/avg/min/max).
     * @param {string} series
     * @param {'count'|'sum'|'avg'|'min'|'max'} func
     * @param {string} from
     * @param {string} to
     */
    aggregateTimeseries(series, func, from = null, to = null) {
        const s = ffi().aggregate_timeseries(this._handle, series, func, from || '', to || '');
        return s ? JSON.parse(s) : null;
    }

    dropTimeseries(name) {
        if (ffi().drop_timeseries(this._handle, name) !== 0) throw _err(this._handle, 'dropTimeseries');
    }
    listTimeseries() {
        const s = ffi().list_timeseries(this._handle);
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
        const id = ffi().insert_vector(
            this._handle,
            collection,
            JSON.stringify(meta),
            JSON.stringify(vec)
        );
        if (!id) throw _err(this._handle, 'insertVector');
        return id;
    }

    /**
     * Find the top-k nearest neighbours.
     * @param {string}   collection  - Collection name
     * @param {number[]} vec         - Query embedding
     * @param {number}   [topK=10]   - Number of results
     * @param {string}   [metric]    - 'cosine' (default) | 'euclidean' | 'dot'
     * @returns {{ id, score, metadata }[]}
     */
    vectorSearch(collection, vec, topK = 10, metric = 'cosine') {
        const s = ffi().vector_search(this._handle, collection, JSON.stringify(vec), topK, metric);
        return s ? JSON.parse(s) : [];
    }

    dropVectorIndex(name) {
        if (ffi().drop_vector_index(this._handle, name) !== 0) throw _err(this._handle, 'dropVectorIndex');
    }
    listVectorIndexes() {
        const s = ffi().list_vector_indexes(this._handle);
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
        const id = ffi().create_node(this._handle, typeName, JSON.stringify(props));
        if (!id) throw _err(this._handle, 'createNode');
        return id;
    }

    /**
     * Create a directed edge between two nodes.
     * @param {string} typeName  - Edge type e.g. 'KNOWS'
     * @param {string} fromId    - Source node id
     * @param {string} toId      - Target node id
     * @param {object} [props]
     */
    createEdge(typeName, fromId, toId, props = {}) {
        const r = ffi().create_edge(this._handle, typeName, fromId, toId, JSON.stringify(props));
        if (r !== 0) throw _err(this._handle, 'createEdge');
    }

    /**
     * Traverse the graph from a starting node (BFS/DFS).
     * @param {string} startId
     * @param {'outbound'|'inbound'|'any'} direction
     * @param {number} maxDepth
     * @returns {object[]}
     */
    graphTraverse(startId, direction = 'outbound', maxDepth = 3) {
        const s = ffi().graph_traverse(this._handle, startId, direction, maxDepth);
        return s ? JSON.parse(s) : [];
    }

    /**
     * Find the shortest path between two nodes.
     * @param {string} fromId
     * @param {string} toId
     * @returns {string[]} Array of node ids representing the path
     */
    shortestPath(fromId, toId) {
        const s = ffi().shortest_path(this._handle, fromId, toId);
        return s ? JSON.parse(s) : [];
    }

    deleteNode(nodeId) {
        if (ffi().delete_node(this._handle, nodeId) !== 0) throw _err(this._handle, 'deleteNode');
    }
    listNodes(typeName) {
        const s = ffi().list_nodes(this._handle, typeName);
        return s ? JSON.parse(s) : [];
    }

    // ══════════════════════════════════════════════════════════════════════════
    // STREAMING ENGINE
    // ══════════════════════════════════════════════════════════════════════════

    /**
     * Create a streaming topic (message queue).
     * @param {string} topic
     * @param {object} [opts] - { partitions, retention_secs }
     */
    createTopic(topic, opts = {}) {
        const r = ffi().create_topic(this._handle, topic, JSON.stringify(opts));
        if (r !== 0) throw _err(this._handle, 'createTopic');
    }

    /**
     * Publish a message to a topic.
     * @param {string} topic
     * @param {string|object} message
     */
    publish(topic, message) {
        const msg = typeof message === 'string' ? message : JSON.stringify(message);
        const r = ffi().publish(this._handle, topic, msg);
        if (r !== 0) throw _err(this._handle, 'publish');
    }

    /**
     * Subscribe to a topic. Returns subscription id.
     * @param {string} topic
     * @param {string} [consumerGroup]
     * @returns {BigInt} subscription id
     */
    subscribe(topic, consumerGroup = '') {
        const id = ffi().subscribe(this._handle, topic, consumerGroup);
        if (!id) throw _err(this._handle, 'subscribe');
        return id;
    }

    /**
     * Poll messages from a subscription.
     * @param {BigInt} subId - From subscribe()
     * @param {number} maxMsgs
     * @returns {object[]}
     */
    poll(subId, maxMsgs = 100) {
        const s = ffi().poll(this._handle, subId, maxMsgs);
        return s ? JSON.parse(s) : [];
    }

    /**
     * Acknowledge messages up to an offset.
     * @param {BigInt} subId
     * @param {BigInt} offset
     */
    commitOffset(subId, offset) {
        ffi().commit_offset(this._handle, subId, offset);
    }

    unsubscribe(subId) { ffi().unsubscribe(this._handle, subId); }
    dropTopic(topic) {
        if (ffi().drop_topic(this._handle, topic) !== 0) throw _err(this._handle, 'dropTopic');
    }
    listTopics() {
        const s = ffi().list_topics(this._handle);
        return s ? JSON.parse(s) : [];
    }
}

module.exports = { OverdriveDb, IsolationLevel };
