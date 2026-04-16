/**
 * OverDrive-DB — Transactions Example (Node.js)
 *
 * Demonstrates: callback pattern (sync and async), manual pattern,
 * retry helper, and isolation levels.
 *
 * Install:
 *   npm install overdrive-db
 *
 * Run:
 *   node transactions.js
 */
const { OverDrive, TransactionError } = require('overdrive-db');

async function main() {
    const db = OverDrive.open('transactions.odb');

    // Seed some data
    db.insert('accounts', { id: 'acc_alice', owner: 'Alice', balance: 1000 });
    db.insert('accounts', { id: 'acc_bob',   owner: 'Bob',   balance: 500 });
    console.log('Initial balances:');
    db.findAll('accounts').forEach(acc => console.log(`  ${acc.owner}: $${acc.balance}`));

    // ── Callback Pattern (v1.4 — recommended) ──────────────────────────────
    // The callback receives the transaction ID.
    // On success → auto-commit. On exception → auto-rollback.
    console.log('\n=== Callback Transaction (sync) ===');

    const result = db.transaction((txn) => {
        db.updateMany('accounts', "id = 'acc_alice'", { balance: 900 });
        db.updateMany('accounts', "id = 'acc_bob'",   { balance: 600 });
        return 'transferred $100 from Alice to Bob';
    });
    console.log('Result:', result);

    // ── Async Callback ──────────────────────────────────────────────────────
    // transaction() supports async callbacks — returns a Promise.
    console.log('\n=== Async Callback Transaction ===');

    const asyncResult = await db.transaction(async (txn) => {
        const orderId = db.insert('orders', { item: 'widget', qty: 3, status: 'pending' });
        db.insert('audit_log', { event: 'order_created', orderId });
        // Simulate async work (e.g. calling an external service)
        await new Promise(resolve => setTimeout(resolve, 10));
        return orderId;
    });
    console.log('Created order:', asyncResult);

    // ── Rollback on Exception ───────────────────────────────────────────────
    console.log('\n=== Automatic Rollback on Error ===');
    const countBefore = db.count('orders');

    try {
        await db.transaction(async (txn) => {
            db.insert('orders', { item: 'gadget', qty: 1 });
            throw new Error('Something went wrong — transaction will be rolled back');
        });
    } catch (e) {
        console.log('Caught expected error:', e.message);
    }

    const countAfter = db.count('orders');
    console.log(`Orders before: ${countBefore}, after: ${countAfter} (unchanged — rolled back)`);

    // ── Retry Helper ────────────────────────────────────────────────────────
    // transactionWithRetry() retries on TransactionError (e.g. deadlock)
    // with exponential backoff. Other exceptions propagate immediately.
    console.log('\n=== Transaction with Retry ===');

    const retryResult = await db.transactionWithRetry(async (txn) => {
        db.insert('events', { type: 'page_view', url: '/home' });
        return 'event logged';
    }, OverDrive.READ_COMMITTED, 3);
    console.log('Retry result:', retryResult);

    // ── Isolation Levels ────────────────────────────────────────────────────
    console.log('\n=== Isolation Levels ===');

    // READ_COMMITTED (default) — no dirty reads
    const accounts = db.transaction(
        (txn) => db.findAll('accounts'),
        OverDrive.READ_COMMITTED
    );
    console.log('READ_COMMITTED:', accounts.length, 'accounts');

    // SERIALIZABLE — full serial isolation
    const snapshotCount = db.transaction((txn) => {
        const count = db.countWhere('accounts', 'balance > 0');
        db.insert('snapshots', { accountCount: count });
        return count;
    }, OverDrive.SERIALIZABLE);
    console.log('SERIALIZABLE:', snapshotCount, 'accounts with positive balance');

    // ── Manual Pattern (v1.3 — still fully supported) ───────────────────────
    console.log('\n=== Manual Transaction (v1.3 style) ===');
    const txnId = db.beginTransaction();
    try {
        db.insert('logs', { event: 'manual_txn_test', status: 'started' });
        db.commitTransaction(txnId);
        console.log('Manual transaction committed');
    } catch (e) {
        db.abortTransaction(txnId);
        console.log('Manual transaction rolled back:', e.message);
    }

    // Final state
    console.log('\n=== Final Balances ===');
    db.findAll('accounts').forEach(acc => console.log(`  ${acc.owner}: $${acc.balance}`));

    db.close();
    console.log('\nDone!');
}

main().catch(err => {
    console.error('Error:', err.message);
    process.exit(1);
});
