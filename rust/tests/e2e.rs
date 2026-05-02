/// End-to-End Integration Tests — OverDrive-DB Rust SDK v2.2.0
/// Instance name: odb (convention throughout)
/// Run: cargo test --test e2e -- --nocapture

use overdrive::{OverdriveDb, IsolationLevel};
use serde_json::json;
use std::path::Path;

fn tmp(name: &str) -> String {
    let dir = std::env::temp_dir().join("overdrive_sdk_e2e");
    std::fs::create_dir_all(&dir).ok();
    dir.join(format!("{}.odb", name)).to_string_lossy().to_string()
}
fn cleanup(p: &str) { let _ = std::fs::remove_file(p); let _ = std::fs::remove_file(format!("{}.wal", p)); }

// ── TEST 1 ───────────────────────────────────────────────────────────────────
#[test]
fn test_01_open_creates_odb_file() {
    let path = tmp("t01_open");
    cleanup(&path);
    let odb = OverdriveDb::open(&path).expect("open must succeed");
    odb.close().ok();
    assert!(Path::new(&path).exists(), "❌ .odb file not created");
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(size > 0, "❌ .odb file is 0 bytes");
    println!("✅ TEST 1 — .odb created ({} bytes)", size);
    cleanup(&path);
}

// ── TEST 2 ───────────────────────────────────────────────────────────────────
#[test]
fn test_02_insert_get_roundtrip() {
    let path = tmp("t02_crud");
    cleanup(&path);
    let mut odb = OverdriveDb::open(&path).unwrap();
    odb.create_table("users").unwrap();
    let id = odb.insert("users", &json!({"name":"Karthikeyan","role":"engineer","age":28})).unwrap();
    assert!(!id.is_empty(), "❌ empty _id");
    let doc = odb.get("users", &id).unwrap().unwrap();
    assert_eq!(doc["name"], "Karthikeyan");
    assert_eq!(doc["role"], "engineer");
    assert_eq!(doc["age"],  28);
    println!("✅ TEST 2 — odb.insert+get: name={}", doc["name"]);
    odb.close().ok();
    cleanup(&path);
}

// ── TEST 3 ───────────────────────────────────────────────────────────────────
#[test]
fn test_03_count_accurate() {
    let path = tmp("t03_count");
    cleanup(&path);
    let mut odb = OverdriveDb::open(&path).unwrap();
    odb.create_table("items").unwrap();
    assert_eq!(odb.count("items").unwrap(), 0);
    odb.insert("items", &json!({"n":"A"})).unwrap();
    odb.insert("items", &json!({"n":"B"})).unwrap();
    odb.insert("items", &json!({"n":"C"})).unwrap();
    assert_eq!(odb.count("items").unwrap(), 3, "❌ count must be 3");
    println!("✅ TEST 3 — odb.count=3");
    odb.close().ok();
    cleanup(&path);
}

// ── TEST 4 ───────────────────────────────────────────────────────────────────
#[test]
fn test_04_multi_get_fields() {
    let path = tmp("t04_multi");
    cleanup(&path);
    let mut odb = OverdriveDb::open(&path).unwrap();
    odb.create_table("products").unwrap();
    let id1 = odb.insert("products", &json!({"name":"Apple","price":10})).unwrap();
    let id2 = odb.insert("products", &json!({"name":"Banana","price":5})).unwrap();
    let id3 = odb.insert("products", &json!({"name":"Cherry","price":25})).unwrap();
    assert_eq!(odb.get("products",&id1).unwrap().unwrap()["name"], "Apple");
    assert_eq!(odb.get("products",&id2).unwrap().unwrap()["name"], "Banana");
    assert_eq!(odb.get("products",&id3).unwrap().unwrap()["name"], "Cherry");
    println!("✅ TEST 4 — odb.get per-doc fields correct");
    odb.close().ok();
    cleanup(&path);
}

// ── TEST 5 ───────────────────────────────────────────────────────────────────
#[test]
fn test_05_update_changes_field() {
    let path = tmp("t05_update");
    cleanup(&path);
    let mut odb = OverdriveDb::open(&path).unwrap();
    odb.create_table("cfg").unwrap();
    let id = odb.insert("cfg", &json!({"key":"theme","val":"light"})).unwrap();
    assert!(odb.update("cfg", &id, &json!({"val":"dark"})).unwrap());
    assert_eq!(odb.get("cfg",&id).unwrap().unwrap()["val"], "dark");
    println!("✅ TEST 5 — odb.update: light→dark");
    odb.close().ok();
    cleanup(&path);
}

// ── TEST 6 ───────────────────────────────────────────────────────────────────
#[test]
fn test_06_delete_removes_doc() {
    let path = tmp("t06_delete");
    cleanup(&path);
    let mut odb = OverdriveDb::open(&path).unwrap();
    odb.create_table("logs").unwrap();
    let id1 = odb.insert("logs",&json!({"msg":"e1"})).unwrap();
    let id2 = odb.insert("logs",&json!({"msg":"e2"})).unwrap();
    assert!(odb.delete("logs",&id1).unwrap());
    assert_eq!(odb.count("logs").unwrap(), 1);
    assert!(odb.get("logs",&id1).unwrap().is_none());
    assert!(odb.get("logs",&id2).unwrap().is_some());
    println!("✅ TEST 6 — odb.delete: count=1, deleted=None");
    odb.close().ok();
    cleanup(&path);
}

// ── TEST 7 ───────────────────────────────────────────────────────────────────
#[test]
fn test_07_persist_after_reopen() {
    let path = tmp("t07_persist");
    cleanup(&path);
    let stored_id;
    {
        let mut odb = OverdriveDb::open(&path).unwrap();
        odb.create_table("sessions").unwrap();
        stored_id = odb.insert("sessions",&json!({"token":"abc123","user":"afot"})).unwrap();
        odb.sync().ok();
        odb.close().ok();
    }
    {
        let odb = OverdriveDb::open(&path).unwrap();
        assert_eq!(odb.count("sessions").unwrap(), 1, "❌ data must persist");
        let doc = odb.get("sessions",&stored_id).unwrap().unwrap();
        assert_eq!(doc["token"], "abc123");
        assert_eq!(doc["user"],  "afot");
        println!("✅ TEST 7 — odb persisted: token={}", doc["token"]);
        odb.close().ok();
    }
    cleanup(&path);
}

// ── TEST 8 ───────────────────────────────────────────────────────────────────
#[test]
fn test_08_insert_batch() {
    let path = tmp("t08_batch");
    cleanup(&path);
    let mut odb = OverdriveDb::open(&path).unwrap();
    odb.create_table("orders").unwrap();
    let ids = odb.insert_batch("orders", &[
        json!({"order_id":"ORD-001","amount":150}),
        json!({"order_id":"ORD-002","amount":200}),
        json!({"order_id":"ORD-003","amount":75}),
    ]).unwrap();
    assert_eq!(ids.len(), 3);
    assert_eq!(odb.count("orders").unwrap(), 3);
    for id in &ids { assert!(odb.get("orders",id).unwrap().is_some()); }
    println!("✅ TEST 8 — odb.insert_batch: {:?}", ids);
    odb.close().ok();
    cleanup(&path);
}

// ── TEST 9 ───────────────────────────────────────────────────────────────────
#[test]
fn test_09_table_exists() {
    let path = tmp("t09_tables");
    cleanup(&path);
    let mut odb = OverdriveDb::open(&path).unwrap();
    assert_eq!(odb.table_exists("ghost").unwrap(), false);
    odb.create_table("real").unwrap();
    assert_eq!(odb.table_exists("real").unwrap(), true);
    println!("✅ TEST 9 — odb.table_exists: ghost=false, real=true");
    odb.close().ok();
    cleanup(&path);
}

// ── TEST 10 ──────────────────────────────────────────────────────────────────
#[test]
fn test_10_version() {
    let v = OverdriveDb::version();
    assert!(!v.is_empty(), "❌ version empty");
    assert_ne!(v, "unknown", "❌ native lib not loaded");
    assert_eq!(v, "2.2.0", "❌ expected v2.2.0, got {}", v);
    println!("✅ TEST 10 — odb version: {}", v);
}
