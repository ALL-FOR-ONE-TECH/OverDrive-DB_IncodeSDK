'use strict';

/**
 * OverDrive-DB Node.js SDK — Real End-to-End Tests
 *
 * These tests ACTUALLY create .odb files using the native library.
 * A passing test proves the native lib loaded and the full stack works.
 *
 * Run: node test/e2e.js
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');
const os = require('os');

const { OverDrive } = require('../index.js');

// ── Test helpers ────────────────────────────────────────────────────────────

const TEST_DIR = path.join(os.tmpdir(), 'overdrive_e2e_nodejs');
fs.mkdirSync(TEST_DIR, { recursive: true });

function dbPath(name) {
    return path.join(TEST_DIR, `${name}.odb`);
}

function cleanup(p) {
    try { fs.unlinkSync(p); } catch (_) {}
    try { fs.unlinkSync(p + '.wal'); } catch (_) {}
}

let passed = 0;
let failed = 0;

function test(name, fn) {
    try {
        fn();
        console.log(`  ✅ ${name}`);
        passed++;
    } catch (err) {
        console.error(`  ❌ ${name}`);
        console.error(`     ${err.message}`);
        failed++;
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

console.log('\n🔶 OverDrive-DB Node.js SDK — End-to-End Tests\n');

// TEST 1: Open creates a real .odb file on disk
test('open() creates a real .odb file on disk', () => {
    const p = dbPath('open_creates');
    cleanup(p);

    const db = OverDrive.open(p);
    db.close();

    assert.ok(fs.existsSync(p), `❌ .odb file was NOT created at: ${p}`);
    const size = fs.statSync(p).size;
    assert.ok(size > 0, `❌ .odb file exists but is 0 bytes`);
    console.log(`     → .odb created (${size} bytes)`);
    cleanup(p);
});

// TEST 2: Insert → get roundtrip
test('insert() + get() roundtrip', () => {
    const p = dbPath('crud_roundtrip');
    cleanup(p);

    const db = OverDrive.open(p);
    db.createTable('users');

    const id = db.insert('users', { name: 'Karthikeyan', role: 'engineer', age: 28 });
    assert.ok(id && id.length > 0, '❌ insert must return a non-empty _id');

    const doc = db.get('users', id);
    assert.ok(doc !== null, '❌ document must exist after insert');
    assert.strictEqual(doc.name, 'Karthikeyan', '❌ name mismatch');
    assert.strictEqual(doc.role, 'engineer', '❌ role mismatch');
    assert.strictEqual(doc.age, 28, '❌ age mismatch');

    console.log(`     → _id: ${id}, name: ${doc.name}`);
    db.close();
    cleanup(p);
});

// TEST 3: Count is accurate
test('count() returns correct number', () => {
    const p = dbPath('count_check');
    cleanup(p);

    const db = OverDrive.open(p);
    db.createTable('items');

    assert.strictEqual(db.count('items'), 0, '❌ empty table count must be 0');

    db.insert('items', { name: 'A' });
    db.insert('items', { name: 'B' });
    db.insert('items', { name: 'C' });

    const count = db.count('items');
    assert.strictEqual(count, 3, `❌ count must be 3 after 3 inserts, got ${count}`);
    console.log(`     → count: ${count}`);
    db.close();
    cleanup(p);
});

// TEST 4: get() retrieves correct fields for multiple docs
test('get() retrieves correct fields per document', () => {
    const p = dbPath('multi_get');
    cleanup(p);

    const db = OverDrive.open(p);
    db.createTable('products');

    const id1 = db.insert('products', { name: 'Apple',  price: 10 });
    const id2 = db.insert('products', { name: 'Banana', price: 5  });
    const id3 = db.insert('products', { name: 'Cherry', price: 25 });

    // Verify each document is individually retrievable with correct data
    const apple  = db.get('products', id1);
    const banana = db.get('products', id2);
    const cherry = db.get('products', id3);

    assert.strictEqual(apple.name,  'Apple',  '❌ apple name mismatch');
    assert.strictEqual(banana.name, 'Banana', '❌ banana name mismatch');
    assert.strictEqual(cherry.name, 'Cherry', '❌ cherry name mismatch');
    assert.strictEqual(apple.price,  10, '❌ apple price mismatch');
    assert.strictEqual(cherry.price, 25, '❌ cherry price mismatch');

    // Rust-side filter using count — all 3 inserted
    const total = db.count('products');
    assert.strictEqual(total, 3, `❌ must have 3 products, got ${total}`);

    console.log(`     → apple=${apple.price}, banana=${banana.price}, cherry=${cherry.price}`);
    db.close();
    cleanup(p);
});

// TEST 5: Update changes a field
test('update() changes a field — verified by get()', () => {
    const p = dbPath('update_check');
    cleanup(p);

    const db = OverDrive.open(p);
    db.createTable('config');

    const id = db.insert('config', { key: 'theme', value: 'light' });
    const ok = db.update('config', id, { value: 'dark' });
    assert.ok(ok, '❌ update must return true for existing doc');

    const doc = db.get('config', id);
    assert.strictEqual(doc.value, 'dark', `❌ value must be updated to 'dark', got '${doc.value}'`);

    console.log(`     → theme: '${doc.value}' ✓`);
    db.close();
    cleanup(p);
});

// TEST 6: Delete removes a document
test('delete() removes document — count drops by 1', () => {
    const p = dbPath('delete_check');
    cleanup(p);

    const db = OverDrive.open(p);
    db.createTable('logs');

    const id1 = db.insert('logs', { msg: 'event1' });
    const id2 = db.insert('logs', { msg: 'event2' });
    assert.strictEqual(db.count('logs'), 2);

    const deleted = db.delete('logs', id1);
    assert.ok(deleted, '❌ delete must return true for existing doc');
    assert.strictEqual(db.count('logs'), 1, '❌ count must be 1 after delete');
    assert.strictEqual(db.get('logs', id1), null, '❌ deleted doc must return null');
    assert.ok(db.get('logs', id2) !== null, '❌ other doc must still exist');

    console.log(`     → deleted id1, id2 still present`);
    db.close();
    cleanup(p);
});

// TEST 7: Data persists after close + reopen
test('data persists after close() + open()', () => {
    const p = dbPath('persistence');
    cleanup(p);

    let storedId;

    // Write phase — capture _id for retrieval after reopen
    {
        const db = OverDrive.open(p);
        db.createTable('sessions');
        storedId = db.insert('sessions', { token: 'abc123', user: 'afot_admin' });
        db.sync();
        db.close();
    }

    // Read phase — fresh open, verify via count() + get() by stored _id
    {
        const db = OverDrive.open(p);

        const count = db.count('sessions');
        assert.strictEqual(count, 1, `❌ data must persist after reopen. count=${count}`);

        const doc = db.get('sessions', storedId);
        assert.ok(doc !== null, '❌ document must be retrievable by _id after reopen');
        assert.strictEqual(doc.token, 'abc123',     '❌ token must persist');
        assert.strictEqual(doc.user,  'afot_admin', '❌ user must persist');

        console.log(`     → persisted: token=${doc.token}, _id=${storedId}`);
        db.close();
    }

    cleanup(p);
});

// TEST 8: insertMany inserts all docs
test('insertMany() inserts all docs and count matches', () => {
    const p = dbPath('insert_many');
    cleanup(p);

    const db = OverDrive.open(p);
    db.createTable('orders');

    const ids = db.insertMany('orders', [
        { order_id: 'ORD-001', amount: 150 },
        { order_id: 'ORD-002', amount: 200 },
        { order_id: 'ORD-003', amount: 75  },
    ]);

    assert.strictEqual(ids.length, 3, `❌ insertMany must return 3 IDs, got ${ids.length}`);
    ids.forEach((id, i) => assert.ok(id && id.length > 0, `❌ ID[${i}] must be non-empty`));

    const count = db.count('orders');
    assert.strictEqual(count, 3, `❌ count must be 3 after insertMany`);

    console.log(`     → 3 orders inserted: ${ids.map(id => id.slice(0, 8)).join(', ')}...`);
    db.close();
    cleanup(p);
});

// TEST 9: tableExists returns correct bool
test('tableExists() returns correct bool', () => {
    const p = dbPath('table_exists');
    cleanup(p);

    const db = OverDrive.open(p);
    assert.strictEqual(db.tableExists('ghost_table'), false, '❌ non-existent table must return false');
    db.createTable('real_table');
    assert.strictEqual(db.tableExists('real_table'), true, '❌ created table must return true');

    console.log(`     → tableExists: ghost=false, real_table=true`);
    db.close();
    cleanup(p);
});

// TEST 10: version() is a real version string
test('version() returns a valid version string from native lib', () => {
    const v = OverDrive.version();
    assert.ok(typeof v === 'string' && v.length > 0, '❌ version must be a non-empty string');
    assert.notStrictEqual(v, 'unknown', '❌ version returned "unknown" — native lib may not have loaded');
    console.log(`     → SDK version: ${v}`);
});

// ── Summary ─────────────────────────────────────────────────────────────────

console.log(`\n${'─'.repeat(50)}`);
console.log(`Results: ${passed} passed, ${failed} failed`);
console.log(`${'─'.repeat(50)}\n`);

// Cleanup temp dir
try { fs.rmdirSync(TEST_DIR, { recursive: true }); } catch (_) {}

process.exit(failed > 0 ? 1 : 0);
