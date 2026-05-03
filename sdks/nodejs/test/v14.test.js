/**
 * OverDrive-DB Node.js SDK — Unit Tests (v1.4)
 * Task 44: Tests for all v1.4 features
 *
 * Run: npx jest test/v14.test.js
 */

const {
    OverDrive, OverDriveError, AuthenticationError, TableError,
    QueryError, TransactionError, OverDriveIOError, FFIError,
} = require('../index');
const fs = require('fs');
const path = require('path');

const TEST_DIR = path.join(__dirname, '_test_data');

function testDb(name) { return path.join(TEST_DIR, name); }

beforeAll(() => { fs.mkdirSync(TEST_DIR, { recursive: true }); });
afterAll(() => { fs.rmSync(TEST_DIR, { recursive: true, force: true }); });

// ── Task 44.1: open() with password and engine ─────────

describe('open() method', () => {
    test('opens database with default options', () => {
        const db = OverDrive.open(testDb('default.odb'));
        expect(db).toBeDefined();
        expect(db.path).toBe(testDb('default.odb'));
        db.close();
    });

    test('opens database with engine parameter', () => {
        const db = OverDrive.open(testDb('ram.odb'), { engine: 'RAM' });
        expect(db).toBeDefined();
        db.close();
    });

    test('rejects invalid engine', () => {
        expect(() => {
            OverDrive.open(testDb('bad.odb'), { engine: 'InvalidEngine' });
        }).toThrow(OverDriveError);
    });

    test('rejects short password', () => {
        expect(() => {
            OverDrive.open(testDb('short.odb'), { password: 'abc' });
        }).toThrow(/at least 8 characters/);
    });

    test('opens with password (8+ chars)', () => {
        const db = OverDrive.open(testDb('enc.odb'), { password: 'my-secret-password-123' });
        expect(db).toBeDefined();
        db.close();
    });

    test('autoCreateTables defaults to true', () => {
        const db = OverDrive.open(testDb('auto.odb'));
        db.insert('auto_table', { key: 'value' });
        expect(db.tableExists('auto_table')).toBe(true);
        db.close();
    });
});

// ── Task 44.2: RAM engine methods ──────────────────────

describe('RAM engine methods', () => {
    test('snapshot and restore', () => {
        const db = OverDrive.open(testDb('ram_snap.odb'), { engine: 'RAM' });
        db.insert('cache', { key: 'session', value: 'abc123' });

        const snapPath = testDb('snap.odb');
        db.snapshot(snapPath);
        expect(fs.existsSync(snapPath)).toBe(true);

        db.restore(snapPath);
        db.close();
    });

    test('memoryUsage returns valid structure', () => {
        const db = OverDrive.open(testDb('ram_mem.odb'), { engine: 'RAM' });
        db.insert('data', { key: 'value' });

        const usage = db.memoryUsage();
        expect(usage).toHaveProperty('bytes');
        expect(usage).toHaveProperty('mb');
        expect(usage).toHaveProperty('limit_bytes');
        expect(usage).toHaveProperty('percent');
        expect(typeof usage.bytes).toBe('number');
        db.close();
    });

    test('createTable with RAM engine', () => {
        const db = OverDrive.open(testDb('hybrid.odb'));
        db.createTable('ram_table', { engine: 'RAM' });
        db.insert('ram_table', { key: 'fast' });
        expect(db.count('ram_table')).toBe(1);
        db.close();
    });
});

// ── Task 44.3: watchdog function ───────────────────────

describe('watchdog()', () => {
    test('reports valid database', () => {
        const dbPath = testDb('watchdog.odb');
        const db = OverDrive.open(dbPath);
        db.insert('data', { key: 'value' });
        db.close();

        const report = OverDrive.watchdog(dbPath);
        expect(report.filePath).toBe(dbPath);
        expect(report.fileSizeBytes).toBeGreaterThan(0);
        expect(report.integrityStatus).toBe('valid');
        expect(report.magicValid).toBe(true);
        expect(report.pageCount).toBeGreaterThan(0);
    });

    test('reports missing database', () => {
        const report = OverDrive.watchdog(testDb('nonexistent.odb'));
        expect(report.integrityStatus).toBe('missing');
    });

    test('returns WatchdogReport structure', () => {
        const dbPath = testDb('wd_struct.odb');
        const db = OverDrive.open(dbPath);
        db.close();

        const report = OverDrive.watchdog(dbPath);
        expect(report).toHaveProperty('filePath');
        expect(report).toHaveProperty('fileSizeBytes');
        expect(report).toHaveProperty('lastModified');
        expect(report).toHaveProperty('integrityStatus');
        expect(report).toHaveProperty('corruptionDetails');
        expect(report).toHaveProperty('pageCount');
        expect(report).toHaveProperty('magicValid');
    });
});

// ── Task 44.4: transaction callback ────────────────────

describe('transaction()', () => {
    test('auto-commits on success', () => {
        const db = OverDrive.open(testDb('txn_commit.odb'));
        db.createTable('users');

        const result = db.transaction((txn) => {
            db.insert('users', { name: 'Alice' });
            return 'committed';
        });

        expect(result).toBe('committed');
        expect(db.count('users')).toBe(1);
        db.close();
    });

    test('auto-rolls back on exception', () => {
        const db = OverDrive.open(testDb('txn_rollback.odb'));
        db.createTable('users');

        expect(() => {
            db.transaction((txn) => {
                db.insert('users', { name: 'Alice' });
                throw new Error('test rollback');
            });
        }).toThrow('test rollback');

        db.close();
    });

    test('supports async callbacks', async () => {
        const db = OverDrive.open(testDb('txn_async.odb'));
        db.createTable('async_table');

        const result = await db.transaction(async (txn) => {
            db.insert('async_table', { key: 'async_value' });
            return 'async_done';
        });

        expect(result).toBe('async_done');
        db.close();
    });

    test('returns context object when no callback', () => {
        const db = OverDrive.open(testDb('txn_ctx.odb'));
        db.createTable('ctx_table');

        const txn = db.transaction();
        expect(txn).toHaveProperty('id');
        expect(txn).toHaveProperty('commit');
        expect(txn).toHaveProperty('abort');
        txn.abort();
        db.close();
    });
});

// ── Task 44.5: helper methods ──────────────────────────

describe('helper methods', () => {
    let db;

    beforeAll(() => {
        db = OverDrive.open(testDb('helpers.odb'));
        db.createTable('items');
        db.insert('items', { name: 'Apple', price: 1.50, category: 'fruit' });
        db.insert('items', { name: 'Banana', price: 0.75, category: 'fruit' });
        db.insert('items', { name: 'Carrot', price: 0.50, category: 'vegetable' });
    });

    afterAll(() => { db.close(); });

    test('findOne returns first match', () => {
        const item = db.findOne('items', "category = 'fruit'");
        expect(item).toBeDefined();
        expect(item.category).toBe('fruit');
    });

    test('findOne returns null when no match', () => {
        const item = db.findOne('items', "category = 'meat'");
        expect(item).toBeNull();
    });

    test('findAll returns all matches', () => {
        const items = db.findAll('items', "category = 'fruit'");
        expect(items.length).toBe(2);
    });

    test('updateMany returns affected count', () => {
        const count = db.updateMany('items', "category = 'fruit'", { organic: true });
        expect(count).toBe(2);
    });

    test('countWhere returns correct count', () => {
        const count = db.countWhere('items', "category = 'fruit'");
        expect(count).toBe(2);
    });

    test('exists returns true for existing id', () => {
        const items = db.findAll('items');
        const id = items[0]._id || items[0]['_id'];
        expect(db.exists('items', id)).toBe(true);
    });

    test('exists returns false for missing id', () => {
        expect(db.exists('items', 'nonexistent_999')).toBe(false);
    });
});

// ── Task 44.6: error handling ──────────────────────────

describe('error handling', () => {
    test('OverDriveError is base class', () => {
        const err = new OverDriveError('test');
        expect(err).toBeInstanceOf(Error);
        expect(err.name).toBe('OverDriveError');
    });

    test('AuthenticationError extends OverDriveError', () => {
        const err = new AuthenticationError('bad password', { code: 'ODB-AUTH-001' });
        expect(err).toBeInstanceOf(OverDriveError);
        expect(err.code).toBe('ODB-AUTH-001');
    });

    test('error includes suggestions', () => {
        const err = new OverDriveError('test', {
            code: 'ODB-AUTH-001',
            suggestions: ['Check password', 'Try again'],
        });
        expect(err.suggestions).toEqual(['Check password', 'Try again']);
        expect(err.message).toContain('Check password');
    });

    test('all error classes exist', () => {
        expect(AuthenticationError).toBeDefined();
        expect(TableError).toBeDefined();
        expect(QueryError).toBeDefined();
        expect(TransactionError).toBeDefined();
        expect(OverDriveIOError).toBeDefined();
        expect(FFIError).toBeDefined();
    });
});

// ── Task 44.7: backward compatibility ──────────────────

describe('backward compatibility', () => {
    test('constructor still works', () => {
        const db = new OverDrive(testDb('compat_ctor.odb'));
        expect(db).toBeDefined();
        db.close();
    });

    test('createTable still works explicitly', () => {
        const db = OverDrive.open(testDb('compat_table.odb'));
        db.createTable('explicit');
        expect(db.tableExists('explicit')).toBe(true);
        db.close();
    });

    test('openEncrypted still works', () => {
        process.env.TEST_ODB_KEY = 'test-key-32-chars-exactly-here!!';
        try {
            const db = OverDrive.openEncrypted(testDb('compat_enc.odb'), 'TEST_ODB_KEY');
            db.close();
        } finally {
            delete process.env.TEST_ODB_KEY;
        }
    });

    test('manual transactions still work', () => {
        const db = OverDrive.open(testDb('compat_txn.odb'));
        db.createTable('txn_test');

        const txnId = db.beginTransaction();
        db.insert('txn_test', { key: 'value' });
        db.commitTransaction(txnId);

        expect(db.count('txn_test')).toBe(1);
        db.close();
    });
});

// ── Task 44 (transactionWithRetry) ─────────────────────

describe('transactionWithRetry()', () => {
    test('succeeds on first attempt', async () => {
        const db = OverDrive.open(testDb('retry_ok.odb'));
        db.createTable('retry');

        const result = await db.transactionWithRetry((txn) => {
            db.insert('retry', { key: 'value' });
            return 'ok';
        });

        expect(result).toBe('ok');
        db.close();
    });
});
