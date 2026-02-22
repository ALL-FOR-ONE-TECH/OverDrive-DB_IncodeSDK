package com.afot.overdrive;

import com.sun.jna.*;
import com.sun.jna.ptr.PointerByReference;
import java.util.*;
import com.google.gson.*;
import com.google.gson.reflect.TypeToken;

/**
 * OverDrive InCode SDK — Java Wrapper
 *
 * <p>Embeddable document database — like SQLite for JSON.</p>
 *
 * <pre>{@code
 * OverDrive db = OverDrive.open("myapp.odb");
 * db.createTable("users");
 * String id = db.insert("users", Map.of("name", "Alice", "age", 30));
 * List<Map<String, Object>> results = db.query("SELECT * FROM users WHERE age > 25");
 * db.close();
 * }</pre>
 */
public class OverDrive implements AutoCloseable {

    // ── Native Library Interface ────────────────
    public interface LibOverDrive extends Library {
        LibOverDrive INSTANCE = Native.load("overdrive", LibOverDrive.class);

        Pointer overdrive_open(String path);
        void overdrive_close(Pointer db);
        void overdrive_sync(Pointer db);
        int overdrive_create_table(Pointer db, String name);
        int overdrive_drop_table(Pointer db, String name);
        Pointer overdrive_list_tables(Pointer db);
        int overdrive_table_exists(Pointer db, String name);
        Pointer overdrive_insert(Pointer db, String table, String jsonDoc);
        Pointer overdrive_get(Pointer db, String table, String id);
        int overdrive_update(Pointer db, String table, String id, String jsonUpdates);
        int overdrive_delete(Pointer db, String table, String id);
        int overdrive_count(Pointer db, String table);
        Pointer overdrive_query(Pointer db, String sql);
        Pointer overdrive_search(Pointer db, String table, String text);
        String overdrive_last_error();
        void overdrive_free_string(Pointer s);
        String overdrive_version();
    }

    private static final LibOverDrive LIB = LibOverDrive.INSTANCE;
    private static final Gson GSON = new Gson();

    private Pointer handle;
    private final String path;

    // ── Lifecycle ───────────────────────────────

    private OverDrive(Pointer handle, String path) {
        this.handle = handle;
        this.path = path;
    }

    /**
     * Open (or create) a database.
     *
     * @param path Path to the database file
     * @return OverDrive instance
     * @throws OverDriveException if open fails
     */
    public static OverDrive open(String path) {
        Pointer h = LIB.overdrive_open(path);
        if (h == null || h == Pointer.NULL) {
            throw new OverDriveException("Failed to open database: " + lastError());
        }
        return new OverDrive(h, path);
    }

    /** Close the database and release resources. */
    @Override
    public void close() {
        if (handle != null && handle != Pointer.NULL) {
            LIB.overdrive_close(handle);
            handle = null;
        }
    }

    /** Force sync data to disk. */
    public void sync() {
        ensureOpen();
        LIB.overdrive_sync(handle);
    }

    /** Get the database file path. */
    public String getPath() { return path; }

    /** Get the SDK version. */
    public static String version() { return LIB.overdrive_version(); }

    // ── Tables ──────────────────────────────────

    /** Create a new table. */
    public void createTable(String name) {
        ensureOpen();
        if (LIB.overdrive_create_table(handle, name) != 0) {
            throw new OverDriveException(lastError());
        }
    }

    /** Drop a table and all its data. */
    public void dropTable(String name) {
        ensureOpen();
        if (LIB.overdrive_drop_table(handle, name) != 0) {
            throw new OverDriveException(lastError());
        }
    }

    /** List all tables. */
    public List<String> listTables() {
        ensureOpen();
        Pointer ptr = LIB.overdrive_list_tables(handle);
        if (ptr == null || ptr == Pointer.NULL) {
            throw new OverDriveException(lastError());
        }
        String json = readAndFree(ptr);
        return GSON.fromJson(json, new TypeToken<List<String>>(){}.getType());
    }

    /** Check if a table exists. */
    public boolean tableExists(String name) {
        ensureOpen();
        return LIB.overdrive_table_exists(handle, name) == 1;
    }

    // ── CRUD ────────────────────────────────────

    /**
     * Insert a JSON document. Returns the generated _id.
     *
     * @param table Table name
     * @param doc Document as a Map
     * @return The auto-generated _id
     */
    public String insert(String table, Map<String, Object> doc) {
        ensureOpen();
        String json = GSON.toJson(doc);
        Pointer ptr = LIB.overdrive_insert(handle, table, json);
        if (ptr == null || ptr == Pointer.NULL) {
            throw new OverDriveException(lastError());
        }
        return readAndFree(ptr);
    }

    /**
     * Insert multiple documents. Returns list of _ids.
     */
    public List<String> insertMany(String table, List<Map<String, Object>> docs) {
        List<String> ids = new ArrayList<>();
        for (Map<String, Object> doc : docs) {
            ids.add(insert(table, doc));
        }
        return ids;
    }

    /**
     * Get a document by _id. Returns null if not found.
     */
    public Map<String, Object> get(String table, String id) {
        ensureOpen();
        Pointer ptr = LIB.overdrive_get(handle, table, id);
        if (ptr == null || ptr == Pointer.NULL) return null;
        String json = readAndFree(ptr);
        return GSON.fromJson(json, new TypeToken<Map<String, Object>>(){}.getType());
    }

    /**
     * Update a document by _id. Returns true if updated.
     */
    public boolean update(String table, String id, Map<String, Object> updates) {
        ensureOpen();
        String json = GSON.toJson(updates);
        int result = LIB.overdrive_update(handle, table, id, json);
        if (result == -1) throw new OverDriveException(lastError());
        return result == 1;
    }

    /**
     * Delete a document by _id. Returns true if deleted.
     */
    public boolean delete(String table, String id) {
        ensureOpen();
        int result = LIB.overdrive_delete(handle, table, id);
        if (result == -1) throw new OverDriveException(lastError());
        return result == 1;
    }

    /**
     * Count documents in a table.
     */
    public int count(String table) {
        ensureOpen();
        int result = LIB.overdrive_count(handle, table);
        if (result == -1) throw new OverDriveException(lastError());
        return Math.max(0, result);
    }

    // ── Query ───────────────────────────────────

    /**
     * Execute an SQL query. Returns result rows.
     */
    public List<Map<String, Object>> query(String sql) {
        ensureOpen();
        Pointer ptr = LIB.overdrive_query(handle, sql);
        if (ptr == null || ptr == Pointer.NULL) {
            throw new OverDriveException(lastError());
        }
        String json = readAndFree(ptr);
        Map<String, Object> result = GSON.fromJson(json, new TypeToken<Map<String, Object>>(){}.getType());
        @SuppressWarnings("unchecked")
        List<Map<String, Object>> rows = (List<Map<String, Object>>) result.getOrDefault("rows", new ArrayList<>());
        return rows;
    }

    /**
     * Full-text search.
     */
    public List<Map<String, Object>> search(String table, String text) {
        ensureOpen();
        Pointer ptr = LIB.overdrive_search(handle, table, text);
        if (ptr == null || ptr == Pointer.NULL) return new ArrayList<>();
        String json = readAndFree(ptr);
        return GSON.fromJson(json, new TypeToken<List<Map<String, Object>>>(){}.getType());
    }

    // ── Internal ────────────────────────────────

    private void ensureOpen() {
        if (handle == null || handle == Pointer.NULL) {
            throw new OverDriveException("Database is closed");
        }
    }

    private static String lastError() {
        String err = LIB.overdrive_last_error();
        return err != null ? err : "unknown error";
    }

    private static String readAndFree(Pointer ptr) {
        String s = ptr.getString(0);
        LIB.overdrive_free_string(ptr);
        return s;
    }
}
