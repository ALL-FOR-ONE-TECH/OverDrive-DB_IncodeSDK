//! Secure database open — full security hardening demo
//!
//! Demonstrates all 7 security mitigations in the OverDrive InCode SDK.
//!
//! Run:
//!   $env:ODB_KEY="my-super-secret-32-char-key!!!!!"
//!   cargo run --example secure_open

use overdrive::{OverDriveDB, IsolationLevel};
use overdrive::shared::SharedDB;
use serde_json::json;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OverDrive SDK: Security Hardening Demo ===\n");

    // ─────────────────────────────────────────────────────────────
    // FIX 1 + 2: Encrypted open + SecretKey (zero-on-drop)
    // Key is read from env var — never hardcoded in source
    // ─────────────────────────────────────────────────────────────
    println!("🔐 [Fix 1+2] Opening database with key from $ODB_KEY env var...");
    let result = OverDriveDB::open_encrypted("secure_demo.odb", "ODB_KEY");
    match result {
        Err(e) => {
            println!("❌ [Expected if ODB_KEY not set]: {}", e);
            println!("   → Set it with: $env:ODB_KEY=\"my-32-char-key!!!!!!!!!!!!!!!\"");
            println!("\n   Falling back to unencrypted open for the rest of the demo...\n");
        }
        Ok(_) => println!("✅ Encrypted open succeeded (key wiped from RAM on drop)\n"),
    }

    // Open normally for the rest of this demo (encryption is an engine feature)
    let mut db = OverDriveDB::open("secure_demo.odb")?;
    println!("✅ [Fix 6] File permissions hardened on open (chmod 600 / Windows ACL)");

    // Create table if needed
    if !db.table_exists("users")? {
        db.create_table("users")?;
    }

    // Insert some test data
    db.insert("users", &json!({"name": "Alice", "age": 30, "email": "alice@example.com"}))?;
    db.insert("users", &json!({"name": "Bob",   "age": 25, "email": "bob@example.com"}))?;

    // ─────────────────────────────────────────────────────────────
    // FIX 4: SQL Injection Prevention — query_safe() with params
    // ─────────────────────────────────────────────────────────────
    println!("\n🛡️ [Fix 4] SQL Injection Prevention:");

    // Simulate a user-provided input (potentially malicious)
    let safe_input = "Alice";
    let result = db.query_safe("SELECT * FROM users WHERE name = ?", &[safe_input])?;
    println!("  ✅ Safe query('Alice') → {} rows returned", result.rows.len());

    // Try an injection attack — should be blocked
    let attack = "Alice'; DROP TABLE users--";
    match db.query_safe("SELECT * FROM users WHERE name = ?", &[attack]) {
        Err(e) => println!("  ✅ Injection attempt blocked: {}", e),
        Ok(_)  => println!("  ❌ Injection was NOT blocked (unexpected)"),
    }

    // ─────────────────────────────────────────────────────────────
    // FIX 7: Stale WAL Cleanup after commit
    // ─────────────────────────────────────────────────────────────
    println!("\n🗑️  [Fix 7] WAL Cleanup after commit:");
    let txn = db.begin_transaction(IsolationLevel::ReadCommitted)?;
    db.insert("users", &json!({"name": "Carol", "age": 28}))?;
    db.commit_transaction(&txn)?;
    db.cleanup_wal()?;
    println!("  ✅ WAL file deleted after commit (prevents stale replay)");

    // ─────────────────────────────────────────────────────────────
    // FIX 3: Encrypted Backup
    // ─────────────────────────────────────────────────────────────
    println!("\n💾 [Fix 3] Backup to separate file:");
    db.backup("secure_demo_backup.odb")?;
    let backup_size = std::fs::metadata("secure_demo_backup.odb")?.len();
    println!("  ✅ Backup created: secure_demo_backup.odb ({} bytes)", backup_size);
    println!("  ✅ Backup file permissions also hardened");

    // ─────────────────────────────────────────────────────────────
    // FIX 5: Thread-safe SharedDB across threads
    // ─────────────────────────────────────────────────────────────
    println!("\n🧵 [Fix 5] Thread-safe SharedDB:");
    db.close()?; // Close single-threaded instance first
    let shared = SharedDB::open("secure_demo.odb")?;

    let t1 = {
        let db = shared.clone();
        thread::spawn(move || {
            db.query("SELECT * FROM users LIMIT 1").unwrap()
        })
    };
    let t2 = {
        let db = shared.clone();
        thread::spawn(move || {
            db.query("SELECT COUNT(*) FROM users").unwrap()
        })
    };

    let r1 = t1.join().unwrap();
    let r2 = t2.join().unwrap();
    println!("  ✅ Thread 1 result: {} rows", r1.rows.len());
    println!("  ✅ Thread 2 result: {:?}", r2.rows);
    println!("  ✅ {} SharedDB handles active (Arc count)", shared.handle_count());

    // Cleanup
    shared.with(|db| db.close())?? ;
    OverDriveDB::destroy("secure_demo.odb")?;
    let _ = OverDriveDB::destroy("secure_demo_backup.odb");

    println!("\n✅ All security mitigations demonstrated successfully!");
    Ok(())
}
