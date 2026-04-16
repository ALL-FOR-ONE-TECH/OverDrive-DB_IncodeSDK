/**
 * OverDrive-DB — Watchdog Monitoring Example (Node.js)
 *
 * Demonstrates: integrity checks, health monitoring, pre-open validation,
 * periodic monitoring, and backup verification.
 *
 * Install:
 *   npm install overdrive-db
 *
 * Run:
 *   node watchdog_monitoring.js
 */
const { OverDrive } = require('overdrive-db');

/** Pretty-print a WatchdogReport object. */
function printReport(report) {
    const icons = { valid: '✓', corrupted: '✗', missing: '?' };
    const icon = icons[report.integrityStatus] || '?';
    console.log(`  ${icon} Status:   ${report.integrityStatus}`);
    console.log(`    Path:     ${report.filePath}`);
    console.log(`    Size:     ${report.fileSizeBytes.toLocaleString()} bytes`);
    console.log(`    Modified: ${report.lastModified}`);
    console.log(`    Pages:    ${report.pageCount}`);
    console.log(`    Magic OK: ${report.magicValid}`);
    if (report.corruptionDetails) {
        console.log(`    Details:  ${report.corruptionDetails}`);
    }
}

/** Sleep for ms milliseconds. */
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
    // ── Create a database to monitor ───────────────────────────────────────
    const db = OverDrive.open('monitored.odb');
    for (let i = 0; i < 10; i++) {
        db.insert('metrics', { sensor: `sensor_${i}`, value: i * 1.5, unit: '°C' });
    }
    db.close();

    // ── Basic Integrity Check ───────────────────────────────────────────────
    // watchdog() is a static method — no open database handle required.
    // Completes in < 100ms for files under 1 GB.
    console.log('=== Basic Integrity Check ===');
    let report = OverDrive.watchdog('monitored.odb');
    printReport(report);

    if (report.integrityStatus === 'valid') {
        console.log('  → Safe to open');
        const db2 = OverDrive.open('monitored.odb');
        console.log(`  → Opened successfully, ${db2.count('metrics')} records`);
        db2.close();
    }

    // ── Missing File ────────────────────────────────────────────────────────
    console.log('\n=== Missing File ===');
    report = OverDrive.watchdog('nonexistent.odb');
    printReport(report);

    // ── Pre-Open Validation Pattern ─────────────────────────────────────────
    // Always check integrity before opening in production.
    console.log('\n=== Pre-Open Validation Pattern ===');

    function safeOpen(dbPath, options = {}) {
        const check = OverDrive.watchdog(dbPath);
        if (check.integrityStatus === 'missing') {
            // New database — safe to create
            console.log(`  Creating new database: ${dbPath}`);
            return OverDrive.open(dbPath, options);
        } else if (check.integrityStatus === 'corrupted') {
            throw new Error(
                `Database ${dbPath} is corrupted: ${check.corruptionDetails}\n` +
                'Restore from backup before opening.'
            );
        } else {
            console.log(`  Opening healthy database: ${dbPath} (${check.fileSizeBytes.toLocaleString()} bytes)`);
            return OverDrive.open(dbPath, options);
        }
    }

    const safeDb = safeOpen('monitored.odb');
    safeDb.close();

    // ── Backup Verification ─────────────────────────────────────────────────
    console.log('\n=== Backup Verification ===');
    // Create a backup
    const backupDb = OverDrive.open('monitored.odb');
    backupDb.backup('monitored_backup.odb');
    backupDb.close();

    // Verify the backup is intact before trusting it
    const backupReport = OverDrive.watchdog('monitored_backup.odb');
    printReport(backupReport);
    if (backupReport.integrityStatus === 'valid') {
        console.log('  → Backup verified — safe to use for restore');
    } else {
        console.log('  → Backup is invalid — do not use for restore!');
    }

    // ── Periodic Monitoring ─────────────────────────────────────────────────
    // Run watchdog on a schedule to detect corruption early.
    console.log('\n=== Periodic Monitoring (3 checks, 1s apart) ===');
    for (let i = 1; i <= 3; i++) {
        const r = OverDrive.watchdog('monitored.odb');
        const icon = r.integrityStatus === 'valid' ? '✓' : '✗';
        console.log(`  Check ${i}: ${icon} ${r.integrityStatus} ` +
            `(${r.fileSizeBytes.toLocaleString()} bytes, ${r.pageCount} pages)`);
        if (r.integrityStatus !== 'valid') {
            console.log(`    ALERT: ${r.corruptionDetails}`);
        }
        await sleep(1000);
    }

    // ── Monitor Multiple Databases ──────────────────────────────────────────
    console.log('\n=== Monitor Multiple Databases ===');
    const databases = ['monitored.odb', 'monitored_backup.odb', 'nonexistent.odb'];
    databases.forEach(dbPath => {
        const r = OverDrive.watchdog(dbPath);
        const icons = { valid: '✓', corrupted: '✗', missing: '?' };
        console.log(`  ${icons[r.integrityStatus] || '?'} ${dbPath}: ${r.integrityStatus}`);
    });

    console.log('\nDone!');
}

main().catch(err => {
    console.error('Error:', err.message);
    process.exit(1);
});
