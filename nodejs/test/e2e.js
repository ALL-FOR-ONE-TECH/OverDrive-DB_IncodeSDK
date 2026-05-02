'use strict';
/**
 * OverDrive-DB Node.js SDK — End-to-End Tests v2.2.0
 * Instance name: odb (convention throughout)
 * Run: node test/e2e.js
 */

const assert = require('assert');
const path   = require('path');
const fs     = require('fs');
const os     = require('os');
const { OverdriveDb } = require('../index.js');

const TEST_DIR = path.join(os.tmpdir(), 'overdrive_sdk_e2e_node');
fs.mkdirSync(TEST_DIR, { recursive: true });

function dbPath(name) { return path.join(TEST_DIR, `${name}.odb`); }
function cleanup(p) {
    try { fs.unlinkSync(p); } catch (_) {}
    try { fs.unlinkSync(p + '.wal'); } catch (_) {}
}

let passed = 0, failed = 0;
function test(name, fn) {
    try { fn(); console.log(`  ✅ ${name}`); passed++; }
    catch (e) { console.error(`  ❌ ${name}\n     ${e.message}`); failed++; }
}

console.log('\n🔶 OverDrive-DB Node.js SDK — E2E Tests v2.2.0\n');

// TEST 1 — open() creates .odb file
test('open() creates .odb file on disk', () => {
    const p = dbPath('t01_open');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    odb.close();
    assert.ok(fs.existsSync(p), '❌ .odb not created');
    assert.ok(fs.statSync(p).size > 0, '❌ file is 0 bytes');
    console.log(`     → ${fs.statSync(p).size} bytes`);
    cleanup(p);
});

// TEST 2 — insert() + get() roundtrip
test('odb.insert() + odb.get() roundtrip', () => {
    const p = dbPath('t02_crud');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    odb.createTable('users');
    const id = odb.insert('users', { name: 'Karthikeyan', role: 'engineer', age: 28 });
    assert.ok(id && id.length > 0, '❌ empty _id');
    const doc = odb.get('users', id);
    assert.ok(doc !== null, '❌ doc not found');
    assert.strictEqual(doc.name, 'Karthikeyan');
    assert.strictEqual(doc.role, 'engineer');
    assert.strictEqual(doc.age,  28);
    console.log(`     → _id: ${id}, name: ${doc.name}`);
    odb.close();
    cleanup(p);
});

// TEST 3 — count() accurate
test('odb.count() accurate (0→3)', () => {
    const p = dbPath('t03_count');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    odb.createTable('items');
    assert.strictEqual(odb.count('items'), 0);
    odb.insert('items', { n: 'A' });
    odb.insert('items', { n: 'B' });
    odb.insert('items', { n: 'C' });
    assert.strictEqual(odb.count('items'), 3, '❌ count must be 3');
    console.log(`     → count: 3`);
    odb.close();
    cleanup(p);
});

// TEST 4 — multi get per-doc fields
test('odb.get() returns correct fields per document', () => {
    const p = dbPath('t04_multi');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    odb.createTable('products');
    const id1 = odb.insert('products', { name: 'Apple',  price: 10 });
    const id2 = odb.insert('products', { name: 'Banana', price: 5  });
    const id3 = odb.insert('products', { name: 'Cherry', price: 25 });
    assert.strictEqual(odb.get('products', id1).name, 'Apple');
    assert.strictEqual(odb.get('products', id2).name, 'Banana');
    assert.strictEqual(odb.get('products', id3).name, 'Cherry');
    assert.strictEqual(odb.count('products'), 3);
    console.log(`     → apple=10, banana=5, cherry=25`);
    odb.close();
    cleanup(p);
});

// TEST 5 — update()
test('odb.update() changes field verified by get()', () => {
    const p = dbPath('t05_update');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    odb.createTable('config');
    const id = odb.insert('config', { key: 'theme', value: 'light' });
    assert.ok(odb.update('config', id, { value: 'dark' }), '❌ update must return true');
    assert.strictEqual(odb.get('config', id).value, 'dark');
    console.log(`     → theme: light → dark`);
    odb.close();
    cleanup(p);
});

// TEST 6 — delete()
test('odb.delete() removes doc, count drops', () => {
    const p = dbPath('t06_delete');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    odb.createTable('logs');
    const id1 = odb.insert('logs', { msg: 'e1' });
    const id2 = odb.insert('logs', { msg: 'e2' });
    assert.ok(odb.delete('logs', id1), '❌ delete must return true');
    assert.strictEqual(odb.count('logs'), 1);
    assert.strictEqual(odb.get('logs', id1), null, '❌ deleted doc must be null');
    assert.ok(odb.get('logs', id2) !== null, '❌ remaining doc must exist');
    console.log(`     → deleted id1, id2 still present`);
    odb.close();
    cleanup(p);
});

// TEST 7 — persist after close + reopen
test('data persists after odb.close() + OverdriveDb.open()', () => {
    const p = dbPath('t07_persist');
    cleanup(p);
    let storedId;
    {
        const odb = OverdriveDb.open(p);
        odb.createTable('sessions');
        storedId = odb.insert('sessions', { token: 'abc123', user: 'afot_admin' });
        odb.sync();
        odb.close();
    }
    {
        const odb = OverdriveDb.open(p);
        assert.strictEqual(odb.count('sessions'), 1, '❌ data must persist');
        const doc = odb.get('sessions', storedId);
        assert.ok(doc !== null, '❌ doc must exist after reopen');
        assert.strictEqual(doc.token, 'abc123');
        assert.strictEqual(doc.user,  'afot_admin');
        console.log(`     → persisted: token=${doc.token}, _id=${storedId}`);
        odb.close();
    }
    cleanup(p);
});

// TEST 8 — insertMany()
test('odb.insertMany() inserts all, count matches', () => {
    const p = dbPath('t08_batch');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    odb.createTable('orders');
    const ids = odb.insertMany('orders', [
        { order_id: 'ORD-001', amount: 150 },
        { order_id: 'ORD-002', amount: 200 },
        { order_id: 'ORD-003', amount: 75  },
    ]);
    assert.strictEqual(ids.length, 3, '❌ must return 3 IDs');
    assert.strictEqual(odb.count('orders'), 3);
    ids.forEach(id => assert.ok(odb.get('orders', id) !== null));
    console.log(`     → 3 orders: ${ids.join(', ')}`);
    odb.close();
    cleanup(p);
});

// TEST 9 — tableExists()
test('odb.tableExists() returns correct bool', () => {
    const p = dbPath('t09_tables');
    cleanup(p);
    const odb = OverdriveDb.open(p);
    assert.strictEqual(odb.tableExists('ghost'), false);
    odb.createTable('real');
    assert.strictEqual(odb.tableExists('real'), true);
    console.log(`     → ghost=false, real=true`);
    odb.close();
    cleanup(p);
});

// TEST 10 — version()
test('OverdriveDb.version() returns 2.2.0', () => {
    const v = OverdriveDb.version();
    assert.ok(v && v.length > 0, '❌ version empty');
    assert.notStrictEqual(v, 'unknown', '❌ native lib not loaded');
    assert.strictEqual(v, '2.2.0', `❌ expected 2.2.0, got ${v}`);
    console.log(`     → version: ${v}`);
});

// ── Summary ──────────────────────────────────────────────────────────────────
console.log(`\n${'─'.repeat(50)}`);
console.log(`Results: ${passed} passed, ${failed} failed`);
console.log(`${'─'.repeat(50)}\n`);
try { fs.rmSync(TEST_DIR, { recursive: true }); } catch (_) {}
process.exit(failed > 0 ? 1 : 0);
