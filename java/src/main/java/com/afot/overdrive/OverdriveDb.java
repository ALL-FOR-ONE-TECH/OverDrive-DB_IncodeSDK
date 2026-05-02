package com.afot.overdrive;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.jna.Pointer;

import java.io.IOException;
import java.util.List;
import java.util.Map;

/**
 * OverDrive-DB Java SDK v2.2.0
 *
 * <p>Idiomatic usage with {@code odb} as the instance name:
 * <pre>{@code
 * try (OverdriveDb odb = OverdriveDb.open("app.odb")) {
 *     odb.createTable("users");
 *     String id = odb.insert("users", Map.of("name","Alice","age",30));
 *     Map<String,Object> doc = odb.get("users", id);
 * }
 * }</pre>
 */
public class OverdriveDb implements AutoCloseable {

    private static final ObjectMapper JSON = new ObjectMapper();
    private static final NativeLib    LIB  = NativeLib.INSTANCE;

    private Pointer handle;

    // ── Isolation levels ──────────────────────────────────────────────────────
    public static final int READ_UNCOMMITTED = 0;
    public static final int READ_COMMITTED   = 1;
    public static final int REPEATABLE_READ  = 2;
    public static final int SERIALIZABLE     = 3;

    private OverdriveDb(Pointer handle) {
        this.handle = handle;
    }

    // ── Static factory ────────────────────────────────────────────────────────

    /**
     * Open or create a database at {@code path}.
     */
    public static OverdriveDb open(String path) {
        Pointer h = LIB.overdrive_open(path);
        if (h == null) throw new RuntimeException("[overdrive] open failed: " + LIB.overdrive_last_error());
        return new OverdriveDb(h);
    }

    /**
     * Open with engine selection and optional password.
     */
    public static OverdriveDb open(String path, String engine, String password) {
        String opts = toJson(Map.of(
                "password", password != null ? password : "",
                "auto_create_tables", true));
        Pointer h = LIB.overdrive_open_with_engine(path, engine, opts);
        if (h == null) throw new RuntimeException("[overdrive] open failed: " + LIB.overdrive_last_error());
        return new OverdriveDb(h);
    }

    /**
     * Return native library version string.
     */
    public static String version() {
        return LIB.overdrive_version();
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────────

    @Override
    public void close() {
        if (handle != null) {
            LIB.overdrive_close(handle);
            handle = null;
        }
    }

    /** Flush pending writes to disk. */
    public void sync() {
        LIB.overdrive_sync(handle);
    }

    // ── Tables ────────────────────────────────────────────────────────────────

    /** Create a table. */
    public void createTable(String name) {
        if (LIB.overdrive_create_table(handle, name) != 0)
            throw new RuntimeException("[overdrive] createTable: " + LIB.overdrive_last_error());
    }

    /** Drop a table. */
    public void dropTable(String name) {
        if (LIB.overdrive_drop_table(handle, name) != 0)
            throw new RuntimeException("[overdrive] dropTable: " + LIB.overdrive_last_error());
    }

    /** List all table names. */
    @SuppressWarnings("unchecked")
    public List<String> listTables() {
        Pointer ptr = LIB.overdrive_list_tables(handle);
        if (ptr == null) return List.of();
        return fromJson(readFree(ptr), List.class);
    }

    /** Return true if the table exists. */
    public boolean tableExists(String name) {
        return LIB.overdrive_table_exists(handle, name) == 1;
    }

    // ── CRUD ──────────────────────────────────────────────────────────────────

    /**
     * Insert a document. Returns the generated {@code _id}.
     */
    public String insert(String table, Map<String, Object> doc) {
        Pointer ptr = LIB.overdrive_insert(handle, table, toJson(doc));
        if (ptr == null) throw new RuntimeException("[overdrive] insert: " + LIB.overdrive_last_error());
        return readFree(ptr);
    }

    /**
     * Insert multiple documents. Returns list of generated {@code _id}s.
     */
    public List<String> insertMany(String table, List<Map<String, Object>> docs) {
        return docs.stream()
                   .map(doc -> insert(table, doc))
                   .toList();
    }

    /**
     * Get a document by {@code _id}. Returns {@code null} if not found.
     */
    @SuppressWarnings("unchecked")
    public Map<String, Object> get(String table, String id) {
        Pointer ptr = LIB.overdrive_get(handle, table, id);
        if (ptr == null) return null;
        return fromJson(readFree(ptr), Map.class);
    }

    /**
     * Update a document. Returns {@code true} if found and updated.
     */
    public boolean update(String table, String id, Map<String, Object> patch) {
        int r = LIB.overdrive_update(handle, table, id, toJson(patch));
        if (r < 0) throw new RuntimeException("[overdrive] update: " + LIB.overdrive_last_error());
        return r == 1;
    }

    /**
     * Delete a document by {@code _id}. Returns {@code true} if deleted.
     */
    public boolean delete(String table, String id) {
        int r = LIB.overdrive_delete(handle, table, id);
        if (r < 0) throw new RuntimeException("[overdrive] delete: " + LIB.overdrive_last_error());
        return r == 1;
    }

    /**
     * Count documents in a table.
     */
    public int count(String table) {
        int n = LIB.overdrive_count(handle, table);
        if (n < 0) throw new RuntimeException("[overdrive] count: " + LIB.overdrive_last_error());
        return n;
    }

    // ── Query ─────────────────────────────────────────────────────────────────

    /**
     * Execute a SQL query. Returns rows as a list of maps.
     */
    @SuppressWarnings("unchecked")
    public List<Map<String, Object>> query(String sql) {
        Pointer ptr = LIB.overdrive_query(handle, sql);
        if (ptr == null) throw new RuntimeException("[overdrive] query: " + LIB.overdrive_last_error());
        String s = readFree(ptr);
        try {
            Map<String, Object> result = JSON.readValue(s, new TypeReference<>() {});
            if (result.containsKey("rows")) {
                return (List<Map<String, Object>>) result.get("rows");
            }
            return List.of(result);
        } catch (IOException e) {
            throw new RuntimeException("[overdrive] query parse error: " + e.getMessage());
        }
    }

    /**
     * Full-text search across a table.
     */
    @SuppressWarnings("unchecked")
    public List<Map<String, Object>> search(String table, String text) {
        Pointer ptr = LIB.overdrive_search(handle, table, text);
        if (ptr == null) return List.of();
        return fromJson(readFree(ptr), List.class);
    }

    // ── Transactions ──────────────────────────────────────────────────────────

    /** Begin an MVCC transaction. Returns the transaction ID. */
    public long beginTransaction(int isolationLevel) {
        long id = LIB.overdrive_begin_transaction(handle, isolationLevel);
        if (id == 0) throw new RuntimeException("[overdrive] beginTransaction: " + LIB.overdrive_last_error());
        return id;
    }

    /** Commit a transaction. */
    public void commitTransaction(long txnId) {
        if (LIB.overdrive_commit_transaction(handle, txnId) != 0)
            throw new RuntimeException("[overdrive] commit: " + LIB.overdrive_last_error());
    }

    /** Abort (rollback) a transaction. */
    public void abortTransaction(long txnId) {
        LIB.overdrive_abort_transaction(handle, txnId);
    }

    /**
     * Run a callback inside a transaction.
     * Auto-commits on success, auto-aborts on exception.
     */
    @FunctionalInterface
    public interface TxnCallback {
        void run(OverdriveDb odb) throws Exception;
    }

    public void transaction(TxnCallback callback) {
        transaction(callback, READ_COMMITTED);
    }

    public void transaction(TxnCallback callback, int isolationLevel) {
        long txnId = beginTransaction(isolationLevel);
        try {
            callback.run(this);
            commitTransaction(txnId);
        } catch (Exception e) {
            abortTransaction(txnId);
            throw new RuntimeException("[overdrive] transaction rolled back: " + e.getMessage(), e);
        }
    }

    // ── Integrity ─────────────────────────────────────────────────────────────

    @SuppressWarnings("unchecked")
    public Map<String, Object> verifyIntegrity() {
        Pointer ptr = LIB.overdrive_verify_integrity(handle);
        if (ptr == null) throw new RuntimeException("[overdrive] verifyIntegrity: " + LIB.overdrive_last_error());
        return fromJson(readFree(ptr), Map.class);
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    private String readFree(Pointer ptr) {
        String s = ptr.getString(0);
        LIB.overdrive_free_string(ptr);
        return s;
    }

    private static String toJson(Object obj) {
        try { return JSON.writeValueAsString(obj); }
        catch (IOException e) { throw new RuntimeException(e); }
    }

    @SuppressWarnings("unchecked")
    private static <T> T fromJson(String s, Class<?> type) {
        try { return (T) JSON.readValue(s, type); }
        catch (IOException e) { throw new RuntimeException("[overdrive] JSON parse: " + e.getMessage(), e); }
    }
}
