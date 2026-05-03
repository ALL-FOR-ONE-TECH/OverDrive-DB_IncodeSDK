/**
 * OverDrive-DB — Password-Protected Database Example (Node.js)
 *
 * Demonstrates: creating encrypted databases, opening with correct/wrong
 * passwords, password validation, and environment variable best practices.
 *
 * Install:
 *   npm install overdrive-db
 *
 * Run:
 *   node password_protected.js
 */
const { OverDrive, AuthenticationError, OverDriveError } = require('overdrive-db');

// ── Best practice: load password from environment variable ──────────────────
// Never hardcode passwords in source code.
// Set with: $env:DB_PASSWORD = "my-secret-password-123"  (PowerShell)
//       or: export DB_PASSWORD="my-secret-password-123"  (bash)
const PASSWORD = process.env.DB_PASSWORD || 'my-secret-password-123';

async function main() {
    // ── Create an encrypted database ───────────────────────────────────────
    console.log('=== Creating Encrypted Database ===');
    // The password is used to derive an AES-256-GCM key via Argon2id.
    // The salt is stored in the file header — you only need the password to re-open.
    let db = OverDrive.open('secure.odb', { password: PASSWORD });

    db.insert('secrets', { key: 'api_token',     value: 'sk-abc123xyz' });
    db.insert('secrets', { key: 'db_password',   value: 'postgres-secret' });
    db.insert('secrets', { key: 'webhook_secret', value: 'whsec-xyz789' });

    console.log(`Stored ${db.count('secrets')} secrets in encrypted database`);
    db.close();
    console.log('Database closed.');

    // ── Re-open with the correct password ──────────────────────────────────
    console.log('\n=== Re-opening with Correct Password ===');
    db = OverDrive.open('secure.odb', { password: PASSWORD });
    const secrets = db.findAll('secrets');
    console.log(`Retrieved ${secrets.length} secrets:`);
    secrets.forEach(s => {
        // Show key names but mask values in output
        console.log(`  ${s.key}: ${'*'.repeat(String(s.value).length)}`);
    });
    db.close();

    // ── Wrong password is rejected ──────────────────────────────────────────
    console.log('\n=== Wrong Password (Expected Rejection) ===');
    try {
        db = OverDrive.open('secure.odb', { password: 'wrong-password-xyz!' });
        console.log('ERROR: Should not reach here!');
        db.close();
    } catch (e) {
        if (e instanceof AuthenticationError) {
            console.log(`Correctly rejected with AuthenticationError: ${e.code}`);
        } else {
            console.log(`Correctly rejected: ${e.message}`);
        }
    }

    // ── Password too short is rejected ─────────────────────────────────────
    console.log('\n=== Short Password Validation ===');
    try {
        db = OverDrive.open('short_pass.odb', { password: 'short' });
        console.log('ERROR: Should not reach here!');
        db.close();
    } catch (e) {
        console.log(`Correctly rejected (< 8 chars): ${e.message}`);
    }

    // ── Password + engine combination ──────────────────────────────────────
    console.log('\n=== Encrypted RAM Database ===');
    // You can combine password protection with any storage engine
    const ramDb = OverDrive.open('secure_cache.odb', { password: PASSWORD, engine: 'RAM' });
    ramDb.insert('sessions', { userId: 42, token: 'tok_abc', expires: '2026-12-31' });
    console.log(`Encrypted RAM sessions: ${ramDb.count('sessions')}`);
    ramDb.snapshot('secure_cache_snapshot.odb');
    console.log('Snapshot saved.');
    ramDb.close();

    // ── Disable auto-table creation for strict mode ─────────────────────────
    console.log('\n=== Strict Mode (autoCreateTables: false) ===');
    const strictDb = OverDrive.open('strict.odb', {
        password: PASSWORD,
        autoCreateTables: false,
    });
    strictDb.createTable('allowed_table');
    strictDb.insert('allowed_table', { data: 'ok' });
    console.log('Insert into pre-created table: OK');

    try {
        strictDb.insert('unknown_table', { data: 'should fail' });
        console.log('ERROR: Should not reach here!');
    } catch (e) {
        console.log(`Correctly rejected insert into unknown table: ${e.constructor.name}`);
    }
    strictDb.close();

    console.log('\nDone!');
}

main().catch(err => {
    console.error('Error:', err.message);
    process.exit(1);
});
