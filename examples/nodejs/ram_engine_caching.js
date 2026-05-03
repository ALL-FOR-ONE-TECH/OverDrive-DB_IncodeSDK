/**
 * OverDrive-DB — RAM Engine Caching Example (Node.js)
 *
 * Demonstrates: full RAM database, per-table RAM storage, snapshot/restore,
 * memory usage monitoring, and rate limiting patterns.
 *
 * Install:
 *   npm install overdrive-db
 *
 * Run:
 *   node ram_engine_caching.js
 */
const { OverDrive } = require('overdrive-db');

async function main() {
    // ── Full RAM Database ───────────────────────────────────────────────────
    // engine: 'RAM' stores everything in memory — sub-microsecond reads.
    // The .odb path is used for snapshots; no disk I/O during normal operation.
    console.log('=== Full RAM Database ===');
    let cache = OverDrive.open('cache.odb', { engine: 'RAM' });

    // Populate session cache
    for (let i = 1; i <= 100; i++) {
        cache.insert('sessions', {
            userId:  i,
            token:   `tok_${String(i).padStart(4, '0')}`,
            ip:      `10.0.0.${i % 255}`,
            expires: '2026-12-31T23:59:59Z',
        });
    }
    console.log(`Cached ${cache.count('sessions')} sessions`);

    // Sub-microsecond read benchmark
    const start = process.hrtime.bigint();
    for (let i = 0; i < 1000; i++) {
        cache.findOne('sessions', 'userId = 50');
    }
    const elapsedMs = Number(process.hrtime.bigint() - start) / 1e6;
    console.log(`1000 reads in ${elapsedMs.toFixed(2)} ms (${(elapsedMs).toFixed(3)} µs avg)`);

    // ── Memory Usage Monitoring ─────────────────────────────────────────────
    console.log('\n=== Memory Usage ===');
    const usage = cache.memoryUsage();
    console.log(`  Used:    ${usage.bytes.toLocaleString()} bytes (${usage.mb.toFixed(3)} MB)`);
    console.log(`  Limit:   ${usage.limit_bytes.toLocaleString()} bytes`);
    console.log(`  Percent: ${usage.percent.toFixed(2)}%`);

    // ── Snapshot — Persist RAM to Disk ─────────────────────────────────────
    console.log('\n=== Snapshot (RAM → Disk) ===');
    cache.snapshot('./cache_snapshot.odb');
    console.log('Snapshot saved to ./cache_snapshot.odb');

    // Simulate restart: close and re-open
    cache.close();
    console.log('Cache closed (data would be lost without snapshot)');

    // ── Restore — Load Snapshot Back into RAM ──────────────────────────────
    console.log('\n=== Restore (Disk → RAM) ===');
    const cache2 = OverDrive.open('cache_restored.odb', { engine: 'RAM' });
    cache2.restore('./cache_snapshot.odb');
    console.log(`Restored ${cache2.count('sessions')} sessions from snapshot`);

    // Verify data integrity
    const session = cache2.findOne('sessions', 'userId = 50');
    console.log('Session 50 token:', session ? session.token : 'not found');
    cache2.close();

    // ── Per-Table RAM Storage (Hybrid Mode) ────────────────────────────────
    // Mix RAM tables (hot data) with Disk tables (persistent data) in one DB.
    console.log('\n=== Hybrid: RAM Table + Disk Table ===');
    const db = OverDrive.open('hybrid_app.odb');

    // Hot cache in RAM — fast reads for frequently accessed data
    db.createTable('rate_limits',  { engine: 'RAM' });
    db.createTable('hot_sessions', { engine: 'RAM' });

    // Persistent data on disk (auto-created on first insert)
    db.insert('users',  { name: 'Alice', role: 'admin' });
    db.insert('users',  { name: 'Bob',   role: 'user' });
    db.insert('orders', { user: 'Alice', item: 'widget', qty: 3 });

    // Populate RAM tables
    db.insert('rate_limits',  { key: 'user:1:api', count: 42, windowSec: 60 });
    db.insert('rate_limits',  { key: 'user:2:api', count: 7,  windowSec: 60 });
    db.insert('hot_sessions', { token: 'tok_abc', userId: 1, ttl: 3600 });

    console.log(`  RAM  'rate_limits':  ${db.count('rate_limits')} records`);
    console.log(`  RAM  'hot_sessions': ${db.count('hot_sessions')} records`);
    console.log(`  Disk 'users':        ${db.count('users')} records`);
    console.log(`  Disk 'orders':       ${db.count('orders')} records`);

    // ── Rate Limiting Pattern ───────────────────────────────────────────────
    console.log('\n=== Rate Limiting Pattern ===');

    function checkRateLimit(userId, limit = 100) {
        const key = `user:${userId}:api`;
        const record = db.findOne('rate_limits', `key = '${key}'`);
        if (!record) {
            db.insert('rate_limits', { key, count: 1, windowSec: 60 });
            return true;
        }
        const count = record.count || 0;
        if (count >= limit) return false;
        db.updateMany('rate_limits', `key = '${key}'`, { count: count + 1 });
        return true;
    }

    [1, 2, 3].forEach(userId => {
        const allowed = checkRateLimit(userId);
        console.log(`  User ${userId}: ${allowed ? 'allowed' : 'rate-limited'}`);
    });

    db.close();
    console.log('\nDone!');
}

main().catch(err => {
    console.error('Error:', err.message);
    process.exit(1);
});
