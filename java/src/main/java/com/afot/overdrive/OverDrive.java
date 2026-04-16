package com.afot.overdrive;

import com.sun.jna.*;
import com.sun.jna.ptr.PointerByReference;
import java.util.*;
import com.google.gson.*;
import com.google.gson.reflect.TypeToken;

/**
 * OverDrive InCode SDK — Java Wrapper (v1.4)
 *
 * <p>Embeddable document database — like SQLite for JSON.</p>
 *
 * <pre>{@code
 * // v1.4 simplified API
 * OverDrive db = OverDrive.open("myapp.odb");
 * db.insert("users", Map.of("name", "Alice", "age", 30));  // table auto-created
 * List<Map<String, Object>> results = db.query("SELECT * FROM users WHERE age > 25");
 * db.close();
 *
 * // Password-protected
 * OverDrive db = OverDrive.open("secure.odb", new OpenOptions().password("my-secret-pass"));
 *
 * // RAM engine
 * OverDrive db = OverDrive.open("cache.odb", new OpenOptions().engine("RAM"));
 * }</pre>
 */
public class OverDrive implements AutoCloseable {

    // ── Native Library Interface ────────────────
    public interface LibOverDrive extends Library {
        // Core (v1.3)
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

        // v1.4: Engine & auto-create
        Pointer overdrive_open_with_engine(String path, String engine, String optionsJson);
        int overdrive_set_auto_create_tables(Pointer db, int enabled);
        String overdrive_get_error_details();
        int overdrive_create_table_with_engine(Pointer db, String name, String engine);

        // v1.4: RAM engine
        Pointer overdrive_create_ram_db(String path, long maxMemoryBytes);
        int overdrive_create_ram_table(Pointer db, String tableName);
        int overdrive_snapshot(Pointer db, String snapshotPath);
        int overdrive_restore(Pointer db, String snapshotPath);
        Pointer overdrive_memory_usage(Pointer db);
        int overdrive_set_memory_limit(Pointer db, long maxBytes);

        // v1.4: Watchdog
        Pointer overdrive_watchdog(String path);

        // v1.4: Transactions
        long overdrive_begin_transaction(Pointer db, int isolationLevel);
        int overdrive_commit_transaction(Pointer db, long txnId);
        int overdrive_abort_transaction(Pointer db, long txnId);
    }

    // Lazy-loaded native library holder — avoids class-init failure when
    // the native library is absent (e.g. in unit-test environments).
    private static final class NativeHolder {
        static final LibOverDrive INSTANCE;
        static {
            LibOverDrive lib;
            try {
                lib = Native.load("overdrive", LibOverDrive.class);
            } catch (UnsatisfiedLinkError e) {
                lib = null;
            }
            INSTANCE = lib;
        }
    }

    private static LibOverDrive lib() {
        LibOverDrive l = NativeHolder.INSTANCE;
        if (l == null) {
            throw new OverDriveException(
                "Could not load native library 'overdrive'. " +
                "Ensure the native library is on the library path.",
                "ODB-FFI-001", "",
                Arrays.asList(
                    "Reinstall the package",
                    "Verify your platform is supported (Windows x64, Linux x64/ARM64, macOS x64/ARM64)",
                    "Check that the package installation completed successfully"
                ),
                "https://overdrive-db.com/docs/errors/ODB-FFI-001"
            );
        }
        return l;
    }

    private static final Gson GSON = new Gson();

    private Pointer handle;
    private final String path;

    // ── Isolation Level Constants ───────────────
    public static final int READ_UNCOMMITTED = 0;
    public static final int READ_COMMITTED = 1;
    public static final int REPEATABLE_READ = 2;
    public static final int SERIALIZABLE = 3;

    // ── OpenOptions (Task 19) ──────────────────

    /**
     * Options for opening a database via {@link OverDrive#open(String, OpenOptions)}.
     * Uses a fluent builder pattern.
     *
     * <pre>{@code
     * OverDrive db = OverDrive.open("app.odb", new OpenOptions()
     *     .password("my-secret-pass")
     *     .engine("RAM")
     *     .autoCreateTables(true));
     * }</pre>
     */
    public static class OpenOptions {
        private String password = null;
        private String engine = "Disk";
        private boolean autoCreateTables = true;

        public OpenOptions() {}

        /** Set encryption password (minimum 8 characters). */
        public OpenOptions password(String password) { this.password = password; return this; }
        /** Set storage engine: "Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming". */
        public OpenOptions engine(String engine) { this.engine = engine; return this; }
        /** Enable/disable auto-table creation on first insert (default: true). */
        public OpenOptions autoCreateTables(boolean enabled) { this.autoCreateTables = enabled; return this; }

        public String getPassword() { return password; }
        public String getEngine() { return engine; }
        public boolean isAutoCreateTables() { return autoCreateTables; }
    }

    // ── TableOptions ───────────────────────────

    /**
     * Options for creating a table with a specific engine.
     */
    public static class TableOptions {
        private String engine = "Disk";
        public TableOptions() {}
        public TableOptions engine(String engine) { this.engine = engine; return this; }
        public String getEngine() { return engine; }
    }

    // ── WatchdogReport (Task 21) ───────────────

    /**
     * Result of a watchdog integrity check on a .odb file.
     */
    public static class WatchdogReport {
        private final String filePath;
        private final long fileSizeBytes;
        private final long lastModified;
        private final String integrityStatus;
        private final String corruptionDetails;
        private final int pageCount;
        private final boolean magicValid;

        public WatchdogReport(String filePath, long fileSizeBytes, long lastModified,
                              String integrityStatus, String corruptionDetails,
                              int pageCount, boolean magicValid) {
            this.filePath = filePath;
            this.fileSizeBytes = fileSizeBytes;
            this.lastModified = lastModified;
            this.integrityStatus = integrityStatus;
            this.corruptionDetails = corruptionDetails;
            this.pageCount = pageCount;
            this.magicValid = magicValid;
        }

        public String getFilePath() { return filePath; }
        public long getFileSizeBytes() { return fileSizeBytes; }
        public long getLastModified() { return lastModified; }
        public String getIntegrityStatus() { return integrityStatus; }
        public String getCorruptionDetails() { return corruptionDetails; }
        public int getPageCount() { return pageCount; }
        public boolean isMagicValid() { return magicValid; }

        @Override
        public String toString() {
            return String.format("WatchdogReport{path='%s', size=%d, status='%s', pages=%d, magic=%b}",
                filePath, fileSizeBytes, integrityStatus, pageCount, magicValid);
        }
    }

    // ── MemoryUsage (Task 20) ──────────────────

    /**
     * RAM engine memory consumption statistics.
     */
    public static class MemoryUsage {
        private final long bytes;
        private final double mb;
        private final long limitBytes;
        private final double percent;

        public MemoryUsage(long bytes, double mb, long limitBytes, double percent) {
            this.bytes = bytes;
            this.mb = mb;
            this.limitBytes = limitBytes;
            this.percent = percent;
        }

        public long getBytes() { return bytes; }
        public double getMb() { return mb; }
        public long getLimitBytes() { return limitBytes; }
        public double getPercent() { return percent; }

        @Override
        public String toString() {
            return String.format("MemoryUsage{bytes=%d, mb=%.2f, limit=%d, percent=%.1f%%}",
                bytes, mb, limitBytes, percent);
        }
    }

    // ── TransactionCallback (Task 22) ──────────

    /**
     * Functional interface for the transaction callback pattern.
     *
     * @param <T> Return type of the transaction
     */
    @FunctionalInterface
    public interface TransactionCallback<T> {
        T execute(long txnId) throws Exception;
    }

    // ── Lifecycle ──────────────────────────────

    private OverDrive(Pointer handle, String path) {
        this.handle = handle;
        this.path = path;
    }

    /**
     * Open (or create) a database — legacy constructor.
     * For v1.4 features, use {@link #open(String, OpenOptions)} instead.
     *
     * @param path Path to the database file
     * @return OverDrive instance
     * @throws OverDriveException if open fails
     */
    public static OverDrive open(String path) {
        Pointer h = lib().overdrive_open(path);
        if (h == null || h == Pointer.NULL) {
            checkErrorStructured();
            throw new OverDriveException("Failed to open database: " + path);
        }
        OverDrive db = new OverDrive(h, path);
        setSecurePermissions(path);
        return db;
    }

    /**
     * Open or create a database — the simplified v1.4 entry point.
     *
     * <pre>{@code
     * // Basic
     * OverDrive db = OverDrive.open("app.odb", new OpenOptions());
     *
     * // Password-protected
     * OverDrive db = OverDrive.open("secure.odb", new OpenOptions().password("secret123"));
     *
     * // RAM engine
     * OverDrive db = OverDrive.open("cache.odb", new OpenOptions().engine("RAM"));
     * }</pre>
     *
     * @param path    Path to the database file
     * @param options Open options (password, engine, autoCreateTables)
     * @return OverDrive instance
     * @throws OverDriveException if open fails
     */
    public static OverDrive open(String path, OpenOptions options) {
        if (path == null || path.isEmpty()) path = "./app.odb";
        if (options == null) options = new OpenOptions();

        String[] validEngines = {"Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming"};
        boolean validEngine = false;
        for (String e : validEngines) {
            if (e.equals(options.getEngine())) { validEngine = true; break; }
        }
        if (!validEngine) {
            throw new OverDriveException("Invalid engine '" + options.getEngine() +
                "'. Valid options: Disk, Graph, RAM, Streaming, Time-Series, Vector");
        }

        if (options.getPassword() != null && options.getPassword().length() < 8) {
            throw new OverDriveException("Password must be at least 8 characters long",
                "ODB-AUTH-002", "", Arrays.asList(
                    "Use a password with at least 8 characters",
                    "Consider using 12+ characters for better security"
                ), "https://overdrive-db.com/docs/errors/ODB-AUTH-002");
        }

        // Build options JSON
        Map<String, Object> optObj = new LinkedHashMap<>();
        optObj.put("auto_create_tables", options.isAutoCreateTables());
        if (options.getPassword() != null) {
            optObj.put("password", options.getPassword());
        }
        String optionsJson = GSON.toJson(optObj);

        Pointer h = lib().overdrive_open_with_engine(path, options.getEngine(), optionsJson);
        if (h == null || h == Pointer.NULL) {
            checkErrorStructured();
            throw new OverDriveException("Failed to open database: " + path);
        }

        OverDrive db = new OverDrive(h, path);
        setSecurePermissions(path);
        return db;
    }

    /** Close the database and release resources. */
    @Override
    public void close() {
        if (handle != null && handle != Pointer.NULL) {
            lib().overdrive_close(handle);
            handle = null;
        }
    }

    /** Force sync data to disk. */
    public void sync() {
        ensureOpen();
        lib().overdrive_sync(handle);
    }

    /** Get the database file path. */
    public String getPath() { return path; }

    /** Get the SDK version. */
    public static String version() { return lib().overdrive_version(); }

    // ── Tables ──────────────────────────────────

    /** Create a new table (Disk engine). */
    public void createTable(String name) {
        ensureOpen();
        if (lib().overdrive_create_table(handle, name) != 0) {
            checkErrorStructured();
        }
    }

    /**
     * Create a table with a specific storage engine.
     *
     * <pre>{@code
     * db.createTable("hot_cache", new TableOptions().engine("RAM"));
     * db.createTable("vectors", new TableOptions().engine("Vector"));
     * }</pre>
     *
     * @param name    Table name
     * @param options Table options with engine selection
     */
    public void createTable(String name, TableOptions options) {
        ensureOpen();
        String engine = (options != null) ? options.getEngine() : "Disk";
        if ("Disk".equals(engine)) {
            if (lib().overdrive_create_table(handle, name) != 0) {
                checkErrorStructured();
            }
        } else {
            if (lib().overdrive_create_table_with_engine(handle, name, engine) != 0) {
                checkErrorStructured();
            }
        }
    }

    /** Drop a table and all its data. */
    public void dropTable(String name) {
        ensureOpen();
        if (lib().overdrive_drop_table(handle, name) != 0) {
            checkErrorStructured();
        }
    }

    /** List all tables. */
    public List<String> listTables() {
        ensureOpen();
        Pointer ptr = lib().overdrive_list_tables(handle);
        if (ptr == null || ptr == Pointer.NULL) {
            checkErrorStructured();
            return new ArrayList<>();
        }
        String json = readAndFree(ptr);
        return GSON.fromJson(json, new TypeToken<List<String>>(){}.getType());
    }

    /** Check if a table exists. */
    public boolean tableExists(String name) {
        ensureOpen();
        return lib().overdrive_table_exists(handle, name) == 1;
    }

    // ── CRUD ────────────────────────────────────

    /**
     * Insert a JSON document. Returns the generated _id.
     *
     * @param table Table name
     * @param doc   Document as a Map
     * @return The auto-generated _id
     */
    public String insert(String table, Map<String, Object> doc) {
        ensureOpen();
        String json = GSON.toJson(doc);
        Pointer ptr = lib().overdrive_insert(handle, table, json);
        if (ptr == null || ptr == Pointer.NULL) {
            checkErrorStructured();
            throw new OverDriveException("Insert failed");
        }
        return readAndFree(ptr);
    }

    /** Insert multiple documents. Returns list of _ids. */
    public List<String> insertMany(String table, List<Map<String, Object>> docs) {
        List<String> ids = new ArrayList<>();
        for (Map<String, Object> doc : docs) {
            ids.add(insert(table, doc));
        }
        return ids;
    }

    /** Get a document by _id. Returns null if not found. */
    public Map<String, Object> get(String table, String id) {
        ensureOpen();
        Pointer ptr = lib().overdrive_get(handle, table, id);
        if (ptr == null || ptr == Pointer.NULL) return null;
        String json = readAndFree(ptr);
        return GSON.fromJson(json, new TypeToken<Map<String, Object>>(){}.getType());
    }

    /** Update a document by _id. Returns true if updated. */
    public boolean update(String table, String id, Map<String, Object> updates) {
        ensureOpen();
        String json = GSON.toJson(updates);
        int result = lib().overdrive_update(handle, table, id, json);
        if (result == -1) checkErrorStructured();
        return result == 1;
    }

    /** Delete a document by _id. Returns true if deleted. */
    public boolean delete(String table, String id) {
        ensureOpen();
        int result = lib().overdrive_delete(handle, table, id);
        if (result == -1) checkErrorStructured();
        return result == 1;
    }

    /** Count documents in a table. */
    public int count(String table) {
        ensureOpen();
        int result = lib().overdrive_count(handle, table);
        if (result == -1) checkErrorStructured();
        return Math.max(0, result);
    }

    // ── Query ───────────────────────────────────

    /** Execute an SQL query. Returns result rows. */
    public List<Map<String, Object>> query(String sql) {
        ensureOpen();
        Pointer ptr = lib().overdrive_query(handle, sql);
        if (ptr == null || ptr == Pointer.NULL) {
            checkErrorStructured();
            return new ArrayList<>();
        }
        String json = readAndFree(ptr);
        Map<String, Object> result = GSON.fromJson(json, new TypeToken<Map<String, Object>>(){}.getType());
        @SuppressWarnings("unchecked")
        List<Map<String, Object>> rows = (List<Map<String, Object>>) result.getOrDefault("rows", new ArrayList<>());
        return rows;
    }

    /**
     * Execute an SQL query returning full result with metadata.
     *
     * @return Map with "rows", "columns", "rows_affected" keys
     */
    public Map<String, Object> queryFull(String sql) {
        ensureOpen();
        Pointer ptr = lib().overdrive_query(handle, sql);
        if (ptr == null || ptr == Pointer.NULL) {
            checkErrorStructured();
            Map<String, Object> empty = new LinkedHashMap<>();
            empty.put("rows", new ArrayList<>());
            empty.put("columns", new ArrayList<>());
            empty.put("rows_affected", 0);
            return empty;
        }
        String json = readAndFree(ptr);
        return GSON.fromJson(json, new TypeToken<Map<String, Object>>(){}.getType());
    }

    /** Full-text search. */
    public List<Map<String, Object>> search(String table, String text) {
        ensureOpen();
        Pointer ptr = lib().overdrive_search(handle, table, text);
        if (ptr == null || ptr == Pointer.NULL) return new ArrayList<>();
        String json = readAndFree(ptr);
        return GSON.fromJson(json, new TypeToken<List<Map<String, Object>>>(){}.getType());
    }

    // ── Security (v1.3.0) ───────────────────────

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
        System.setProperty("__OVERDRIVE_KEY", key);
        try {
            return open(path);
        } finally {
            System.clearProperty("__OVERDRIVE_KEY");
            char[] keyChars = key.toCharArray();
            java.util.Arrays.fill(keyChars, '\0');
        }
    }

    /**
     * Create an encrypted backup of the database at destPath.
     * Syncs to disk first, then copies .odb + .wal files.
     */
    public void backup(String destPath) throws java.io.IOException {
        ensureOpen();
        sync();
        java.nio.file.Path src = java.nio.file.Paths.get(path);
        java.nio.file.Path dst = java.nio.file.Paths.get(destPath);
        java.nio.file.Files.copy(src, dst, java.nio.file.StandardCopyOption.REPLACE_EXISTING);
        java.nio.file.Path walSrc = java.nio.file.Paths.get(path + ".wal");
        if (java.nio.file.Files.exists(walSrc)) {
            java.nio.file.Files.copy(walSrc, java.nio.file.Paths.get(destPath + ".wal"),
                java.nio.file.StandardCopyOption.REPLACE_EXISTING);
        }
        setSecurePermissions(destPath);
    }

    /** Delete the WAL file after a confirmed commit. */
    public void cleanupWal() throws java.io.IOException {
        java.nio.file.Path walPath = java.nio.file.Paths.get(path + ".wal");
        java.nio.file.Files.deleteIfExists(walPath);
    }

    /**
     * Execute a parameterized SQL query — SQL injection safe.
     * Use {@code ?} as placeholders; values are sanitized before substitution.
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

    // ── RAM Engine Methods (Task 20) ────────────

    /**
     * Persist the current RAM database to a snapshot file.
     *
     * @param snapshotPath Destination file path
     */
    public void snapshot(String snapshotPath) {
        ensureOpen();
        if (lib().overdrive_snapshot(handle, snapshotPath) != 0) {
            checkErrorStructured();
        }
    }

    /**
     * Load a previously saved snapshot into the current database handle.
     *
     * @param snapshotPath Path to the snapshot file
     */
    public void restore(String snapshotPath) {
        ensureOpen();
        if (lib().overdrive_restore(handle, snapshotPath) != 0) {
            checkErrorStructured();
        }
    }

    /**
     * Return current RAM consumption statistics.
     *
     * @return MemoryUsage with bytes, mb, limit, and percent
     */
    public MemoryUsage memoryUsage() {
        ensureOpen();
        Pointer ptr = lib().overdrive_memory_usage(handle);
        if (ptr == null || ptr == Pointer.NULL) {
            checkErrorStructured();
            return new MemoryUsage(0, 0, 0, 0);
        }
        String json = readAndFree(ptr);
        Map<String, Object> data = GSON.fromJson(json, new TypeToken<Map<String, Object>>(){}.getType());
        return new MemoryUsage(
            ((Number) data.getOrDefault("bytes", 0)).longValue(),
            ((Number) data.getOrDefault("mb", 0)).doubleValue(),
            ((Number) data.getOrDefault("limit_bytes", 0)).longValue(),
            ((Number) data.getOrDefault("percent", 0)).doubleValue()
        );
    }

    // ── Watchdog (Task 21) ──────────────────────

    /**
     * Inspect a .odb file for integrity, size, and modification status.
     * Static method — does not require an open database handle.
     *
     * <pre>{@code
     * WatchdogReport report = OverDrive.watchdog("app.odb");
     * if ("corrupted".equals(report.getIntegrityStatus())) {
     *     System.out.println("Corrupted: " + report.getCorruptionDetails());
     * }
     * }</pre>
     *
     * @param filePath Path to the .odb file to inspect
     * @return WatchdogReport with file health information
     */
    public static WatchdogReport watchdog(String filePath) {
        Pointer ptr = lib().overdrive_watchdog(filePath);
        if (ptr == null || ptr == Pointer.NULL) {
            checkErrorStructured();
            throw new OverDriveException("watchdog() returned NULL for path: " + filePath);
        }
        String json = readAndFree(ptr);
        if (json == null || json.isEmpty()) {
            throw new OverDriveException("watchdog() returned empty response for path: " + filePath);
        }
        Map<String, Object> data = GSON.fromJson(json, new TypeToken<Map<String, Object>>(){}.getType());
        return new WatchdogReport(
            (String) data.getOrDefault("file_path", filePath),
            ((Number) data.getOrDefault("file_size_bytes", 0)).longValue(),
            ((Number) data.getOrDefault("last_modified", 0)).longValue(),
            (String) data.getOrDefault("integrity_status", "missing"),
            (String) data.get("corruption_details"),
            ((Number) data.getOrDefault("page_count", 0)).intValue(),
            Boolean.TRUE.equals(data.get("magic_valid"))
        );
    }

    // ── MVCC Transactions ──────────────────────

    /**
     * Begin a new MVCC transaction.
     *
     * @param isolation Isolation level (use constants like READ_COMMITTED)
     * @return Transaction ID
     */
    public long beginTransaction(int isolation) {
        ensureOpen();
        long txnId = lib().overdrive_begin_transaction(handle, isolation);
        if (txnId == 0) {
            checkErrorStructured();
            throw new OverDriveException.TransactionException(
                "Failed to begin transaction", "ODB-TXN-001", "",
                Collections.emptyList(), "");
        }
        return txnId;
    }

    /** Begin a transaction with default READ_COMMITTED isolation. */
    public long beginTransaction() {
        return beginTransaction(READ_COMMITTED);
    }

    /** Commit a transaction. */
    public void commitTransaction(long txnId) {
        ensureOpen();
        if (lib().overdrive_commit_transaction(handle, txnId) != 0) {
            checkErrorStructured();
        }
    }

    /** Abort (rollback) a transaction. */
    public void abortTransaction(long txnId) {
        ensureOpen();
        if (lib().overdrive_abort_transaction(handle, txnId) != 0) {
            checkErrorStructured();
        }
    }

    // ── Transaction Callback Pattern (Task 22) ──

    /**
     * Execute a block of work inside a transaction with automatic commit/rollback.
     *
     * <pre>{@code
     * String result = db.transaction(txn -> {
     *     db.insert("users", Map.of("name", "Alice"));
     *     db.insert("logs", Map.of("event", "user_created"));
     *     return "done";
     * });
     * }</pre>
     *
     * @param callback  The work to execute inside the transaction
     * @param isolation Isolation level
     * @param <T>       Return type of the callback
     * @return The callback's return value
     */
    public <T> T transaction(TransactionCallback<T> callback, int isolation) {
        long txnId = beginTransaction(isolation);
        try {
            T result = callback.execute(txnId);
            commitTransaction(txnId);
            return result;
        } catch (Exception e) {
            try { abortTransaction(txnId); } catch (Exception ignored) {}
            if (e instanceof RuntimeException) throw (RuntimeException) e;
            throw new OverDriveException("Transaction failed: " + e.getMessage(), e);
        }
    }

    /** Execute a transaction with default READ_COMMITTED isolation. */
    public <T> T transaction(TransactionCallback<T> callback) {
        return transaction(callback, READ_COMMITTED);
    }

    /**
     * Execute a transaction with automatic exponential-backoff retry on TransactionException.
     *
     * @param callback   The work to execute
     * @param isolation  Isolation level
     * @param maxRetries Maximum number of attempts (default: 3)
     * @param <T>        Return type
     * @return The callback's return value
     */
    public <T> T transactionWithRetry(TransactionCallback<T> callback, int isolation, int maxRetries) {
        Exception lastErr = null;
        for (int attempt = 0; attempt < maxRetries; attempt++) {
            try {
                return transaction(callback, isolation);
            } catch (OverDriveException.TransactionException e) {
                lastErr = e;
                if (attempt < maxRetries - 1) {
                    long delay = Math.min(100L * (1L << attempt), 2000L);
                    try { Thread.sleep(delay); } catch (InterruptedException ie) {
                        Thread.currentThread().interrupt();
                        throw new OverDriveException("Transaction retry interrupted", ie);
                    }
                }
            }
        }
        throw (RuntimeException) lastErr;
    }

    /** Execute a transaction with retry, default isolation and 3 retries. */
    public <T> T transactionWithRetry(TransactionCallback<T> callback) {
        return transactionWithRetry(callback, READ_COMMITTED, 3);
    }

    // ── Helper Methods (Task 23) ────────────────

    /**
     * Return the first document matching where, or null if no match.
     *
     * @param table Table name
     * @param where Optional SQL WHERE clause (e.g. "age > 25")
     * @return First matching document or null
     */
    public Map<String, Object> findOne(String table, String where) {
        String sql;
        if (where != null && !where.isEmpty()) {
            sql = "SELECT * FROM " + table + " WHERE " + where + " LIMIT 1";
        } else {
            sql = "SELECT * FROM " + table + " LIMIT 1";
        }
        List<Map<String, Object>> rows = query(sql);
        return rows.isEmpty() ? null : rows.get(0);
    }

    /** Return the first document in the table, or null. */
    public Map<String, Object> findOne(String table) {
        return findOne(table, null);
    }

    /**
     * Return all documents matching where.
     *
     * @param table   Table name
     * @param where   Optional SQL WHERE clause
     * @param orderBy Optional ORDER BY expression
     * @param limit   Max rows (0 = no limit)
     * @return List of matching documents
     */
    public List<Map<String, Object>> findAll(String table, String where, String orderBy, int limit) {
        StringBuilder sql = new StringBuilder("SELECT * FROM ").append(table);
        if (where != null && !where.isEmpty()) sql.append(" WHERE ").append(where);
        if (orderBy != null && !orderBy.isEmpty()) sql.append(" ORDER BY ").append(orderBy);
        if (limit > 0) sql.append(" LIMIT ").append(limit);
        return query(sql.toString());
    }

    /** Return all documents in the table. */
    public List<Map<String, Object>> findAll(String table) {
        return findAll(table, null, null, 0);
    }

    /**
     * Update all documents matching where.
     *
     * @param table   Table name
     * @param where   SQL WHERE clause (required)
     * @param updates Field → new value pairs
     * @return Number of documents updated
     */
    public int updateMany(String table, String where, Map<String, Object> updates) {
        StringBuilder setClauses = new StringBuilder();
        boolean first = true;
        for (Map.Entry<String, Object> entry : updates.entrySet()) {
            if (!first) setClauses.append(", ");
            setClauses.append(entry.getKey()).append(" = ").append(GSON.toJson(entry.getValue()));
            first = false;
        }
        String sql = "UPDATE " + table + " SET " + setClauses + " WHERE " + where;
        Map<String, Object> result = queryFull(sql);
        Object affected = result.get("rows_affected");
        return affected != null ? ((Number) affected).intValue() : 0;
    }

    /**
     * Delete all documents matching where.
     *
     * @param table Table name
     * @param where SQL WHERE clause (required)
     * @return Number of documents deleted
     */
    public int deleteMany(String table, String where) {
        String sql = "DELETE FROM " + table + " WHERE " + where;
        Map<String, Object> result = queryFull(sql);
        Object affected = result.get("rows_affected");
        return affected != null ? ((Number) affected).intValue() : 0;
    }

    /**
     * Count documents matching where.
     *
     * @param table Table name
     * @param where Optional SQL WHERE clause
     * @return Count of matching documents
     */
    public int countWhere(String table, String where) {
        String sql;
        if (where != null && !where.isEmpty()) {
            sql = "SELECT COUNT(*) FROM " + table + " WHERE " + where;
        } else {
            sql = "SELECT COUNT(*) FROM " + table;
        }
        List<Map<String, Object>> rows = query(sql);
        if (rows.isEmpty()) return 0;
        Map<String, Object> row = rows.get(0);
        for (String key : new String[]{"COUNT(*)", "count(*)", "count", "COUNT"}) {
            if (row.containsKey(key)) return ((Number) row.get(key)).intValue();
        }
        // Fallback: return first numeric value
        for (Object val : row.values()) {
            if (val instanceof Number) return ((Number) val).intValue();
        }
        return 0;
    }

    /** Count all documents in a table. */
    public int countWhere(String table) {
        return countWhere(table, null);
    }

    /**
     * Check whether a document with the given id exists.
     *
     * @param table Table name
     * @param id    The _id value
     * @return true if document exists
     */
    public boolean exists(String table, String id) {
        return get(table, id) != null;
    }

    // ── Internal ────────────────────────────────

    private void ensureOpen() {
        if (handle == null || handle == Pointer.NULL) {
            throw new OverDriveException("Database is closed");
        }
    }

    private static String lastError() {
        String err = lib().overdrive_last_error();
        return err != null ? err : "unknown error";
    }

    private static String readAndFree(Pointer ptr) {
        String s = ptr.getString(0);
        lib().overdrive_free_string(ptr);
        return s;
    }

    /**
     * Check for and throw the last error from the native library.
     * Tries overdrive_get_error_details() first for structured JSON,
     * falls back to overdrive_last_error() for a plain string.
     */
    private static void checkErrorStructured() {
        try {
            String detailsRaw = lib().overdrive_get_error_details();
            if (detailsRaw != null && !detailsRaw.isEmpty()) {
                try {
                    Map<String, Object> data = GSON.fromJson(detailsRaw,
                        new TypeToken<Map<String, Object>>(){}.getType());
                    String code = (String) data.getOrDefault("code", "");
                    String msg = (String) data.getOrDefault("message", "");
                    String ctx = (String) data.getOrDefault("context", "");
                    @SuppressWarnings("unchecked")
                    List<String> suggestions = (List<String>) data.getOrDefault("suggestions", Collections.emptyList());
                    String docLink = (String) data.getOrDefault("doc_link", "");

                    if (msg != null && !msg.isEmpty()) {
                        throw makeTypedException(msg, code, ctx, suggestions, docLink);
                    }
                } catch (OverDriveException e) {
                    throw e;
                } catch (Exception ignored) {}
            }
        } catch (OverDriveException e) {
            throw e;
        } catch (Exception ignored) {}

        // Fallback to plain error string
        String err = lib().overdrive_last_error();
        if (err != null && !err.isEmpty()) {
            throw new OverDriveException(err);
        }
    }

    /**
     * Create the correct exception subclass based on the error code prefix.
     */
    private static OverDriveException makeTypedException(String msg, String code,
            String ctx, List<String> suggestions, String docLink) {
        if (code != null && code.startsWith("ODB-AUTH")) {
            return new OverDriveException.AuthenticationException(msg, code, ctx, suggestions, docLink);
        } else if (code != null && code.startsWith("ODB-TABLE")) {
            return new OverDriveException.TableException(msg, code, ctx, suggestions, docLink);
        } else if (code != null && code.startsWith("ODB-QUERY")) {
            return new OverDriveException.QueryException(msg, code, ctx, suggestions, docLink);
        } else if (code != null && code.startsWith("ODB-TXN")) {
            return new OverDriveException.TransactionException(msg, code, ctx, suggestions, docLink);
        } else if (code != null && code.startsWith("ODB-IO")) {
            return new OverDriveException.IOError(msg, code, ctx, suggestions, docLink);
        } else if (code != null && code.startsWith("ODB-FFI")) {
            return new OverDriveException.FFIException(msg, code, ctx, suggestions, docLink);
        }
        return new OverDriveException(msg, code, ctx, suggestions, docLink);
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
