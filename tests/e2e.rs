/// End-to-End Integration Tests for OverDrive InCode SDK
///
/// These tests ACTUALLY open a real .odb file using the native library.
/// Each test creates a database, performs real CRUD, and verifies results.
/// A PASS proves the native lib loaded and the full stack works.
///
/// Run: cargo test --manifest-path x:\OverDrive-DB\IncodeSDK\Cargo.toml --test e2e -- --nocapture

use overdrive::OverDriveDB;
use std::path::Path;

fn tmp_db(name: &str) -> String {
    let dir = std::env::temp_dir().join("overdrive_e2e_tests");
    std::fs::create_dir_all(&dir).ok();
    dir.join(format!("{}.odb", name))
        .to_string_lossy()
        .to_string()
}

fn cleanup(path: &str) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}.wal", path));
}

// ─────────────────────────────────────────────────────────────
// TEST 1: open() creates a real .odb file on disk
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_open_creates_odb_file() {
    let path = tmp_db("t1_open");
    cleanup(&path);

    let db = OverDriveDB::open(&path).expect("open() must succeed — native lib must load");
    db.close().ok();

    assert!(Path::new(&path).exists(),
        "❌ .odb file NOT created at {} — native lib failed to load", path);
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(size > 0, "❌ .odb file is 0 bytes — engine did not initialize");
    println!("✅ TEST 1 PASS — .odb file created ({} bytes)", size);
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 2: insert() → get() roundtrip
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_insert_and_get_roundtrip() {
    let path = tmp_db("t2_crud");
    cleanup(&path);

    let mut db = OverDriveDB::open(&path).expect("open failed");
    db.create_table("users").expect("create_table failed");

    let id = db.insert("users", &serde_json::json!({
        "name": "Karthikeyan", "role": "engineer", "age": 28
    })).expect("insert failed");

    assert!(!id.is_empty(), "❌ insert returned empty _id");
    println!("     → _id: {}", id);

    let doc = db.get("users", &id).expect("get failed").expect("doc not found");
    assert_eq!(doc["name"], "Karthikeyan", "❌ name mismatch");
    assert_eq!(doc["role"], "engineer",    "❌ role mismatch");
    assert_eq!(doc["age"],  28,            "❌ age mismatch");

    println!("✅ TEST 2 PASS — insert+get verified, name={}", doc["name"]);
    db.close().ok();
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 3: count() is accurate
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_count_is_accurate() {
    let path = tmp_db("t3_count");
    cleanup(&path);

    let mut db = OverDriveDB::open(&path).expect("open failed");
    db.create_table("items").expect("create_table failed");
    assert_eq!(db.count("items").unwrap(), 0, "❌ empty table must be 0");

    db.insert("items", &serde_json::json!({"n":"A"})).unwrap();
    db.insert("items", &serde_json::json!({"n":"B"})).unwrap();
    db.insert("items", &serde_json::json!({"n":"C"})).unwrap();

    let c = db.count("items").expect("count failed");
    assert_eq!(c, 3, "❌ count must be 3, got {}", c);
    println!("✅ TEST 3 PASS — count={}", c);
    db.close().ok();
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 4: update() changes a field
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_update_changes_field() {
    let path = tmp_db("t4_update");
    cleanup(&path);

    let mut db = OverDriveDB::open(&path).expect("open failed");
    db.create_table("cfg").expect("create_table failed");

    let id = db.insert("cfg", &serde_json::json!({"key":"theme","val":"light"})).unwrap();
    let ok = db.update("cfg", &id, &serde_json::json!({"val":"dark"})).expect("update failed");
    assert!(ok, "❌ update must return true");

    let doc = db.get("cfg", &id).unwrap().unwrap();
    assert_eq!(doc["val"], "dark", "❌ field must be 'dark' after update");
    println!("✅ TEST 4 PASS — val: light → dark");
    db.close().ok();
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 5: delete() removes document — count drops
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_delete_removes_document() {
    let path = tmp_db("t5_delete");
    cleanup(&path);

    let mut db = OverDriveDB::open(&path).expect("open failed");
    db.create_table("logs").expect("create_table failed");

    let id1 = db.insert("logs", &serde_json::json!({"msg":"e1"})).unwrap();
    let id2 = db.insert("logs", &serde_json::json!({"msg":"e2"})).unwrap();
    assert_eq!(db.count("logs").unwrap(), 2);

    let del = db.delete("logs", &id1).expect("delete failed");
    assert!(del, "❌ delete must return true");
    assert_eq!(db.count("logs").unwrap(), 1, "❌ count must drop to 1");
    assert!(db.get("logs", &id1).unwrap().is_none(), "❌ deleted doc must be None");
    assert!(db.get("logs", &id2).unwrap().is_some(), "❌ remaining doc must exist");

    println!("✅ TEST 5 PASS — delete verified, count=1");
    db.close().ok();
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 6: Data persists after close() + reopen()
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_data_persists_after_reopen() {
    let path = tmp_db("t6_persist");
    cleanup(&path);

    let stored_id;

    // Write phase
    {
        let mut db = OverDriveDB::open(&path).expect("first open failed");
        db.create_table("sessions").expect("create_table failed");
        stored_id = db.insert("sessions", &serde_json::json!({
            "token": "abc123", "user": "afot_admin"
        })).unwrap();
        db.sync().ok();
        db.close().ok();
    }

    // Read phase — fresh open, use count() + get() by known ID
    {
        let db = OverDriveDB::open(&path).expect("second open failed");

        // count() proves data survived close+reopen
        let count = db.count("sessions").expect("count after reopen failed");
        assert_eq!(count, 1, "❌ data must persist. count={} (expected 1)", count);

        // get() by stored ID proves specific document survived
        let doc = db.get("sessions", &stored_id)
            .expect("get after reopen failed")
            .expect("doc must exist after reopen");
        assert_eq!(doc["token"], "abc123",    "❌ token must persist");
        assert_eq!(doc["user"],  "afot_admin","❌ user must persist");

        println!("✅ TEST 6 PASS — data persisted: token={}", doc["token"]);
        db.close().ok();
    }

    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 7: insert_batch() returns correct IDs, count matches
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_batch_insert() {
    let path = tmp_db("t7_batch");
    cleanup(&path);

    let mut db = OverDriveDB::open(&path).expect("open failed");
    db.create_table("orders").expect("create_table failed");

    let docs = vec![
        serde_json::json!({"order_id":"ORD-001","amount":150}),
        serde_json::json!({"order_id":"ORD-002","amount":200}),
        serde_json::json!({"order_id":"ORD-003","amount":75}),
    ];
    let ids = db.insert_batch("orders", &docs).expect("insert_batch failed");
    assert_eq!(ids.len(), 3, "❌ must return 3 IDs, got {}", ids.len());
    for id in &ids { assert!(!id.is_empty(), "❌ ID must not be empty"); }

    assert_eq!(db.count("orders").unwrap(), 3, "❌ count must be 3");

    // Verify each doc can be retrieved by its returned ID
    for (i, id) in ids.iter().enumerate() {
        let doc = db.get("orders", id).unwrap().unwrap();
        assert!(!doc["order_id"].is_null(), "❌ doc[{}] has null order_id", i);
    }
    println!("✅ TEST 7 PASS — {} docs inserted: {:?}", ids.len(), ids);
    db.close().ok();
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 8: table_exists() returns correct bool
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_table_exists() {
    let path = tmp_db("t8_tables");
    cleanup(&path);

    let mut db = OverDriveDB::open(&path).expect("open failed");
    assert_eq!(db.table_exists("ghost").unwrap(), false, "❌ ghost must be false");
    db.create_table("real").expect("create_table failed");
    assert_eq!(db.table_exists("real").unwrap(), true, "❌ real must be true");

    println!("✅ TEST 8 PASS — ghost=false, real=true");
    db.close().ok();
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 9: list_tables() lists created tables
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_list_tables() {
    let path = tmp_db("t9_list");
    cleanup(&path);

    let mut db = OverDriveDB::open(&path).expect("open failed");
    db.create_table("alpha").unwrap();
    db.create_table("beta").unwrap();
    db.create_table("gamma").unwrap();

    let tables = db.list_tables().expect("list_tables failed");
    for name in &["alpha", "beta", "gamma"] {
        assert!(tables.iter().any(|t| t == name),
            "❌ '{}' must appear in list_tables, got: {:?}", name, tables);
    }
    println!("✅ TEST 9 PASS — list_tables: {:?}", tables);
    db.close().ok();
    cleanup(&path);
}

// ─────────────────────────────────────────────────────────────
// TEST 10: version() returns real version from native lib
// ─────────────────────────────────────────────────────────────
#[test]
fn e2e_version_is_valid() {
    let v = OverDriveDB::version();
    assert!(!v.is_empty(), "❌ version() must not be empty");
    assert_ne!(v, "unknown", "❌ 'unknown' means native lib did not load");
    println!("✅ TEST 10 PASS — SDK version: {}", v);
}
