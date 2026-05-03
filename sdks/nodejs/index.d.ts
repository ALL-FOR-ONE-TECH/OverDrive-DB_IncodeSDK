/**
 * TypeScript type definitions for overdrive-db (v1.4)
 */

// ── Error Hierarchy (Task 18) ──────────────────────────────────────────────

export interface OverDriveErrorOptions {
    code?: string;
    context?: string;
    suggestions?: string[];
    docLink?: string;
}

export class OverDriveError extends Error {
    /** Machine-readable error code (e.g. "ODB-AUTH-001"), or "" when not available. */
    code: string;
    /** Additional context string from the native library. */
    context: string;
    /** Actionable suggestions to resolve the error. */
    suggestions: string[];
    /** URL to the relevant documentation page. */
    docLink: string;
    constructor(message?: string, options?: OverDriveErrorOptions);
}

export class AuthenticationError extends OverDriveError {}
export class TableError extends OverDriveError {}
export class QueryError extends OverDriveError {}
export class TransactionError extends OverDriveError {}
export class OverDriveIOError extends OverDriveError {}
export class FFIError extends OverDriveError {}

// ── WatchdogReport (Task 15) ───────────────────────────────────────────────

export interface WatchdogReport {
    /** Path that was inspected. */
    filePath: string;
    /** Size of the .odb file in bytes (0 if missing). */
    fileSizeBytes: number;
    /** Unix timestamp of the last modification (0 if missing). */
    lastModified: number;
    /** Integrity status of the file. */
    integrityStatus: 'valid' | 'corrupted' | 'missing';
    /** Human-readable description of the corruption, or null when healthy. */
    corruptionDetails: string | null;
    /** Number of pages found in the file. */
    pageCount: number;
    /** True when the file's magic number matches the expected value. */
    magicValid: boolean;
}

// ── MemoryUsage (Task 14) ──────────────────────────────────────────────────

export interface MemoryUsage {
    /** Bytes currently used by the RAM engine. */
    bytes: number;
    /** Same value expressed in megabytes. */
    mb: number;
    /** Configured memory limit in bytes. */
    limit_bytes: number;
    /** Utilisation as a percentage of the limit. */
    percent: number;
}

// ── OpenOptions (Task 13) ──────────────────────────────────────────────────

export interface OpenOptions {
    /** Optional password for AES-256-GCM encryption (min 8 characters). */
    password?: string;
    /**
     * Storage engine to use.
     * @default "Disk"
     */
    engine?: 'Disk' | 'RAM' | 'Vector' | 'Time-Series' | 'Graph' | 'Streaming';
    /**
     * Auto-create tables on first insert.
     * @default true
     */
    autoCreateTables?: boolean;
}

// ── CreateTableOptions (Task 14) ───────────────────────────────────────────

export interface CreateTableOptions {
    /** Storage engine for this table. @default "Disk" */
    engine?: 'Disk' | 'RAM' | 'Vector' | 'Time-Series' | 'Graph' | 'Streaming';
}

// ── TransactionContext (Task 16) ───────────────────────────────────────────

export interface TransactionContext {
    /** Transaction ID. */
    id: number;
    /** Commit the transaction. */
    commit(): void;
    /** Abort (rollback) the transaction. */
    abort(): void;
}

// ── OverDrive class ────────────────────────────────────────────────────────

export class OverDrive {
    // ── Isolation level constants ──────────────
    static readonly READ_UNCOMMITTED: 0;
    static readonly READ_COMMITTED: 1;
    static readonly REPEATABLE_READ: 2;
    static readonly SERIALIZABLE: 3;

    /**
     * Open (or create) a database using the legacy constructor.
     * @param dbPath - Path to the database file
     */
    constructor(dbPath: string);

    /** Close the database. */
    close(): void;

    /** Sync data to disk. */
    sync(): void;

    /** Get database file path. */
    readonly path: string;

    /** Get SDK version. */
    static version(): string;

    // ── Simplified API (Task 13) ───────────────

    /**
     * Open or create a database — the simplified v1.4 entry point.
     *
     * @param dbPath - Path to the database file (default: './app.odb')
     * @param options - Open options
     * @returns An open OverDrive instance
     *
     * @example
     * const db = OverDrive.open('myapp.odb');
     * db.insert('users', { name: 'Alice' }); // table auto-created
     *
     * @example
     * const db = OverDrive.open('secure.odb', { password: 'my-secret-pass' });
     *
     * @example
     * const db = OverDrive.open('cache.odb', { engine: 'RAM' });
     */
    static open(dbPath?: string, options?: OpenOptions): OverDrive;

    // ── Tables ─────────────────────────────────

    /**
     * Create a table.
     * @param name - Table name
     * @param options - Table options (engine selection)
     */
    createTable(name: string, options?: CreateTableOptions): void;

    /** Drop a table. */
    dropTable(name: string): void;

    /** List all tables. */
    listTables(): string[];

    /** Check if table exists. */
    tableExists(name: string): boolean;

    // ── CRUD ───────────────────────────────────

    /**
     * Insert a document.
     * @returns The generated _id
     */
    insert(table: string, doc: Record<string, any>): string;

    /**
     * Insert multiple documents.
     * @returns List of _ids
     */
    insertMany(table: string, docs: Record<string, any>[]): string[];

    /**
     * Get a document by _id.
     * @returns The document or null
     */
    get(table: string, id: string): Record<string, any> | null;

    /**
     * Update a document by _id.
     * @returns True if updated
     */
    update(table: string, id: string, updates: Record<string, any>): boolean;

    /**
     * Delete a document by _id.
     * @returns True if deleted
     */
    delete(table: string, id: string): boolean;

    /** Count documents in a table. */
    count(table: string): number;

    // ── Query ──────────────────────────────────

    /** Execute SQL query. */
    query(sql: string): Record<string, any>[];

    /** Execute SQL query with full metadata. */
    queryFull(sql: string): {
        rows: Record<string, any>[];
        columns: string[];
        rows_affected: number;
    };

    /** Full-text search. */
    search(table: string, text: string): Record<string, any>[];

    /** Execute a parameterized SQL query (SQL injection safe). */
    querySafe(sqlTemplate: string, params: string[]): Record<string, any>[];

    // ── Security (v1.3.0) ──────────────────────

    /**
     * Open a database with an encryption key loaded from an environment variable.
     * @param dbPath - Path to the .odb file
     * @param keyEnvVar - Name of the env var holding the key (default: 'ODB_KEY')
     */
    static openEncrypted(dbPath: string, keyEnvVar?: string): OverDrive;

    /** Create an encrypted backup of the database. */
    backup(destPath: string): void;

    /** Delete the WAL file after a confirmed commit. */
    cleanupWal(): void;

    // ── RAM Engine Methods (Task 14) ───────────

    /**
     * Persist the current RAM database to a snapshot file.
     * @param snapshotPath - Destination file path
     */
    snapshot(snapshotPath: string): void;

    /**
     * Load a previously saved snapshot into the current database handle.
     * @param snapshotPath - Path to the snapshot file
     */
    restore(snapshotPath: string): void;

    /**
     * Return current RAM consumption statistics.
     */
    memoryUsage(): MemoryUsage;

    // ── Watchdog (Task 15) ─────────────────────

    /**
     * Inspect a .odb file for integrity, size, and modification status.
     * Static method — does not require an open database handle.
     *
     * @param filePath - Path to the .odb file to inspect
     * @returns WatchdogReport with file health information
     */
    static watchdog(filePath: string): WatchdogReport;

    // ── MVCC Transactions ──────────────────────

    /**
     * Begin a new MVCC transaction.
     * @param isolation - Isolation level (default: READ_COMMITTED)
     * @returns Transaction ID
     */
    beginTransaction(isolation?: number): number;

    /** Commit a transaction. */
    commitTransaction(txnId: number): void;

    /** Abort (rollback) a transaction. */
    abortTransaction(txnId: number): void;

    // ── Transaction Callback Pattern (Task 16) ─

    /**
     * Dual-mode transaction helper.
     *
     * With callback (auto-commit/rollback):
     *   const result = await db.transaction(async (txn) => { ... });
     *
     * Without callback (returns transaction context):
     *   const txn = db.transaction();
     *   txn.commit() / txn.abort()
     *
     * @param callback - Async or sync callback receiving txnId
     * @param isolation - Isolation level (default: READ_COMMITTED)
     */
    transaction(callback: (txnId: number) => Promise<any>, isolation?: number): Promise<any>;
    transaction(callback: (txnId: number) => any, isolation?: number): any;
    transaction(callback?: undefined, isolation?: number): TransactionContext;

    // ── Transaction with Retry (Task 18.5) ─────

    /**
     * Execute a transaction with automatic exponential-backoff retry on TransactionError.
     *
     * @param callback - Async or sync callback receiving txnId
     * @param isolation - Isolation level (default: READ_COMMITTED)
     * @param maxRetries - Maximum number of attempts (default: 3)
     */
    transactionWithRetry(
        callback: (txnId: number) => Promise<any> | any,
        isolation?: number,
        maxRetries?: number
    ): Promise<any>;

    // ── Helper Methods (Task 17) ───────────────

    /**
     * Return the first document matching where, or null if no match.
     * @param table - Table name
     * @param where - Optional SQL WHERE clause (e.g. "age > 25")
     */
    findOne(table: string, where?: string): Record<string, any> | null;

    /**
     * Return all documents matching where.
     * @param table - Table name
     * @param where - Optional SQL WHERE clause
     * @param orderBy - Optional ORDER BY expression
     * @param limit - Max rows (0 = no limit)
     */
    findAll(
        table: string,
        where?: string,
        orderBy?: string,
        limit?: number
    ): Record<string, any>[];

    /**
     * Update all documents matching where.
     * @param table - Table name
     * @param where - SQL WHERE clause (required)
     * @param updates - Field → new value pairs
     * @returns Number of documents updated
     */
    updateMany(table: string, where: string, updates: Record<string, any>): number;

    /**
     * Delete all documents matching where.
     * @param table - Table name
     * @param where - SQL WHERE clause (required)
     * @returns Number of documents deleted
     */
    deleteMany(table: string, where: string): number;

    /**
     * Count documents matching where.
     * @param table - Table name
     * @param where - Optional SQL WHERE clause
     */
    countWhere(table: string, where?: string): number;

    /**
     * Check whether a document with the given id exists.
     * @param table - Table name
     * @param id - The _id value
     */
    exists(table: string, id: string): boolean;
}

// ── SharedOverDrive ────────────────────────────────────────────────────────

export class SharedOverDrive {
    constructor(dbPath: string);
    static openEncrypted(dbPath: string, keyEnvVar?: string): SharedOverDrive;
    query(sql: string): Promise<Record<string, any>[]>;
    querySafe(tmpl: string, params: string[]): Promise<Record<string, any>[]>;
    insert(table: string, doc: Record<string, any>): Promise<string>;
    get(table: string, id: string): Promise<Record<string, any> | null>;
    backup(dest: string): Promise<void>;
    cleanupWal(): Promise<void>;
    sync(): Promise<void>;
    close(): Promise<void>;
}
