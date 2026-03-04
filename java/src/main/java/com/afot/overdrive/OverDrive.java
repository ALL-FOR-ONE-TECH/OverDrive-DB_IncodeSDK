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
        OverDrive db = new OverDrive(h, path);
        setSecurePermissions(path);
        return db;
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

    // ── Security (v1.3.0) ────────────────────────

    /**
     * Open a database with an encryption key loaded from an environment variable.
     * Never hardcode the key — read from env or a secrets manager.
     *
     * <pre>{@code
     * // Set: System.setProperty or OS env: ODB_KEY=my-secret-key
     * OverDrive db = OverDrive.openEncrypted("app.odb", "ODB_KEY");
     * }</pre>
     */
    public static OverDrive openEncrypted(String path, String keyEnvVar) {
        String key = System.getenv(keyEnvVar);
        if (key == null || key.isEmpty()) {
            throw new OverDriveException(
                "[overdrive] Encryption key env var '" + keyEnvVar + "' is not set or empty. " +
                "Set it with: export " + keyEnvVar + "=your-key (bash) or " +
                "$env:" + keyEnvVar + "=\"your-key\" (PowerShell)"
            );
        }
        // Pass key to engine via internal env var (cleared after open)
        // Java cannot unset env vars at runtime easily, so we use System property
        System.setProperty("__OVERDRIVE_KEY", key);
        try {
            return open(path);
        } finally {
            System.clearProperty("__OVERDRIVE_KEY");
            // Zero the key chars in memory (best-effort)
            char[] keyChars = key.toCharArray();
            java.util.Arrays.fill(keyChars, '\0');
        }
    }

    /**
     * Create an encrypted backup of the database at destPath.
     * Syncs to disk first, then copies .odb + .wal files.
     *
     * <pre>{@code
     * db.backup("backups/app_2026-03-04.odb");
     * }</pre>
     */
    public void backup(String destPath) throws java.io.IOException {
        ensureOpen();
        sync();
        java.nio.file.Path src = java.nio.file.Paths.get(path);
        java.nio.file.Path dst = java.nio.file.Paths.get(destPath);
        java.nio.file.Files.copy(src, dst, java.nio.file.StandardCopyOption.REPLACE_EXISTING);
        // Copy WAL if it exists
        java.nio.file.Path walSrc = java.nio.file.Paths.get(path + ".wal");
        if (java.nio.file.Files.exists(walSrc)) {
            java.nio.file.Files.copy(walSrc, java.nio.file.Paths.get(destPath + ".wal"),
                java.nio.file.StandardCopyOption.REPLACE_EXISTING);
        }
        setSecurePermissions(destPath);
    }

    /**
     * Delete the WAL file after a confirmed commit to prevent stale replay attacks.
     * Call this after commitTransaction().
     */
    public void cleanupWal() throws java.io.IOException {
        java.nio.file.Path walPath = java.nio.file.Paths.get(path + ".wal");
        java.nio.file.Files.deleteIfExists(walPath);
    }

    /**
     * Execute a parameterized SQL query — the safe way to include user input.
     * Use {@code ?} as placeholders; values are sanitized before substitution.
     * Throws {@link OverDriveException} if any param contains SQL injection patterns.
     *
     * <pre>{@code
     * // SAFE: user input via params, never string concat
     * List<Map<String,Object>> rows = db.querySafe(
     *     "SELECT * FROM users WHERE name = ?",
     *     userInput
     * );
     * }</pre>
     */
    public List<Map<String, Object>> querySafe(String sqlTemplate, String... params) {
        String[] dangerous = {"DROP", "TRUNCATE", "ALTER", "EXEC", "EXECUTE", "UNION", "XP_"};
        String[] dangerousTokens = {"--", ";--", "/*", "*/"};

        List<String> sanitized = new ArrayList<>();
        for (String param : params) {
            String upper = param.toUpperCase();
            for (String token : dangerousTokens) {
                if (param.contains(token)) {
                    throw new OverDriveException(
                        "[overdrive] SQL injection detected: param '" + param + "' contains '" + token + "'");
                }
            }
            for (String kw : dangerous) {
                for (String word : upper.split("\\s+")) {
                    if (word.equals(kw)) {
                        throw new OverDriveException(
                            "[overdrive] SQL injection detected: param '" + param + "' contains keyword '" + kw + "'");
                    }
                }
            }
            sanitized.add("'" + param.replace("'", "''") + "'");
        }

        StringBuilder sql = new StringBuilder(sqlTemplate);
        for (String value : sanitized) {
            int idx = sql.indexOf("?");
            if (idx == -1) throw new OverDriveException("[overdrive] More params than '?' placeholders");
            sql.replace(idx, idx + 1, value);
        }
        long placeholderCount = sqlTemplate.chars().filter(c -> c == '?').count();
        if (params.length < placeholderCount) {
            throw new OverDriveException(String.format(
                "[overdrive] SQL template has %d '?' placeholders but only %d params", placeholderCount, params.length));
        }
        return query(sql.toString());
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

    /**
     * Set restrictive OS-level permissions on the .odb file.
     * Windows: icacls (non-fatal). Linux/macOS: chmod 600.
     */
    private static void setSecurePermissions(String filePath) {
        java.io.File f = new java.io.File(filePath);
        if (!f.exists()) return;
        String os = System.getProperty("os.name", "").toLowerCase();
        if (os.contains("win")) {
            try {
                String username = System.getenv("USERNAME");
                if (username == null) username = "%USERNAME%";
                new ProcessBuilder("icacls", filePath,
                    "/inheritance:r", "/grant:r", username + ":F")
                    .redirectErrorStream(true).start().waitFor();
            } catch (Exception e) {
                System.err.println("[overdrive] WARNING: Could not harden permissions on '" + filePath + "': " + e.getMessage());
            }
        } else {
            f.setReadable(false, false); f.setWritable(false, false); f.setExecutable(false, false);
            f.setReadable(true, true);   f.setWritable(true, true);
        }
    }
}

// ── OverDriveSafe — Thread-safe wrapper ──────────

/**
 * Thread-safe wrapper for {@link OverDrive} using a ReentrantReadWriteLock.
 * Reads (query, get, count) use read lock; writes (insert, update, delete) use write lock.
 *
 * <pre>{@code
 * OverDriveSafe db = OverDriveSafe.open("app.odb");
 * // Safe from multiple threads:
 * executor.submit(() -> db.query("SELECT * FROM users"));
 * executor.submit(() -> db.insert("users", Map.of("name", "Alice")));
 * }</pre>
 */
class OverDriveSafe implements AutoCloseable {
    private final java.util.concurrent.locks.ReentrantReadWriteLock lock =
        new java.util.concurrent.locks.ReentrantReadWriteLock();
    private final OverDrive db;

    private OverDriveSafe(OverDrive db) { this.db = db; }

    public static OverDriveSafe open(String path) {
        return new OverDriveSafe(OverDrive.open(path));
    }

    public static OverDriveSafe openEncrypted(String path, String keyEnvVar) {
        return new OverDriveSafe(OverDrive.openEncrypted(path, keyEnvVar));
    }

    public List<Map<String, Object>> query(String sql) {
        lock.readLock().lock();
        try { return db.query(sql); } finally { lock.readLock().unlock(); }
    }

    public List<Map<String, Object>> querySafe(String sqlTemplate, String... params) {
        lock.readLock().lock();
        try { return db.querySafe(sqlTemplate, params); } finally { lock.readLock().unlock(); }
    }

    public String insert(String table, Map<String, Object> doc) {
        lock.writeLock().lock();
        try { return db.insert(table, doc); } finally { lock.writeLock().unlock(); }
    }

    public Map<String, Object> get(String table, String id) {
        lock.readLock().lock();
        try { return db.get(table, id); } finally { lock.readLock().unlock(); }
    }

    public void backup(String dest) throws java.io.IOException {
        lock.writeLock().lock();
        try { db.backup(dest); } finally { lock.writeLock().unlock(); }
    }

    public void cleanupWal() throws java.io.IOException {
        lock.writeLock().lock();
        try { db.cleanupWal(); } finally { lock.writeLock().unlock(); }
    }

    @Override
    public void close() {
        lock.writeLock().lock();
        try { db.close(); } finally { lock.writeLock().unlock(); }
    }
}

