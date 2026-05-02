"""OverDrive-DB Python SDK â€” E2E Tests v2.2.0"""
import json
import os
import tempfile
import pytest
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from overdrive import OverdriveDb, IsolationLevel

TMP = os.path.join(tempfile.gettempdir(), "overdrive_sdk_e2e_python")
os.makedirs(TMP, exist_ok=True)

def db_path(name):
    p = os.path.join(TMP, f"{name}.odb")
    for f in [p, p + ".wal"]:
        try: os.remove(f)
        except: pass
    return p

# â”€â”€ TEST 1 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_01_open_creates_file():
    p = db_path("t01_open")
    odb = OverdriveDb.open(p)
    odb.close()
    assert os.path.exists(p), "âŒ .odb not created"
    size = os.path.getsize(p)
    assert size > 0, "âŒ .odb is 0 bytes"
    print(f"\n  â†’ {size} bytes")

# â”€â”€ TEST 2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_02_insert_get_roundtrip():
    p = db_path("t02_crud")
    odb = OverdriveDb.open(p)
    odb.create_table("users")
    id_ = odb.insert("users", {"name": "Karthikeyan", "role": "engineer", "age": 28})
    assert id_ and len(id_) > 0
    doc = odb.get("users", id_)
    assert doc is not None
    assert doc["name"] == "Karthikeyan"
    assert doc["role"] == "engineer"
    assert doc["age"]  == 28
    print(f"\n  â†’ _id={id_}, name={doc['name']}")
    odb.close()

# â”€â”€ TEST 3 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_03_count_accurate():
    p = db_path("t03_count")
    odb = OverdriveDb.open(p)
    odb.create_table("items")
    assert odb.count("items") == 0
    odb.insert("items", {"n": "A"})
    odb.insert("items", {"n": "B"})
    odb.insert("items", {"n": "C"})
    assert odb.count("items") == 3
    print(f"\n  â†’ count=3")
    odb.close()

# â”€â”€ TEST 4 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_04_multi_get_fields():
    p = db_path("t04_multi")
    odb = OverdriveDb.open(p)
    odb.create_table("products")
    id1 = odb.insert("products", {"name": "Apple",  "price": 10})
    id2 = odb.insert("products", {"name": "Banana", "price": 5})
    id3 = odb.insert("products", {"name": "Cherry", "price": 25})
    assert odb.get("products", id1)["name"] == "Apple"
    assert odb.get("products", id2)["name"] == "Banana"
    assert odb.get("products", id3)["name"] == "Cherry"
    print(f"\n  â†’ apple=10, banana=5, cherry=25")
    odb.close()

# â”€â”€ TEST 5 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_05_update_changes_field():
    p = db_path("t05_update")
    odb = OverdriveDb.open(p)
    odb.create_table("cfg")
    id_ = odb.insert("cfg", {"key": "theme", "val": "light"})
    assert odb.update("cfg", id_, {"val": "dark"}) is True
    assert odb.get("cfg", id_)["val"] == "dark"
    print(f"\n  â†’ theme: light â†’ dark")
    odb.close()

# â”€â”€ TEST 6 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_06_delete_removes_doc():
    p = db_path("t06_delete")
    odb = OverdriveDb.open(p)
    odb.create_table("logs")
    id1 = odb.insert("logs", {"msg": "e1"})
    id2 = odb.insert("logs", {"msg": "e2"})
    assert odb.delete("logs", id1) is True
    assert odb.count("logs") == 1
    assert odb.get("logs", id1) is None
    assert odb.get("logs", id2) is not None
    print(f"\n  â†’ deleted id1, id2 still present")
    odb.close()

# â”€â”€ TEST 7 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_07_persist_after_reopen():
    p = db_path("t07_persist")
    odb = OverdriveDb.open(p)
    odb.create_table("sessions")
    stored_id = odb.insert("sessions", {"token": "abc123", "user": "afot"})
    odb.sync()
    odb.close()

    odb2 = OverdriveDb.open(p)
    assert odb2.count("sessions") == 1
    doc = odb2.get("sessions", stored_id)
    assert doc is not None
    assert doc["token"] == "abc123"
    assert doc["user"]  == "afot"
    print(f"\n  â†’ persisted: token={doc['token']}, _id={stored_id}")
    odb2.close()

# â”€â”€ TEST 8 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_08_insert_many():
    p = db_path("t08_batch")
    odb = OverdriveDb.open(p)
    odb.create_table("orders")
    ids = odb.insert_many("orders", [
        {"order_id": "ORD-001", "amount": 150},
        {"order_id": "ORD-002", "amount": 200},
        {"order_id": "ORD-003", "amount": 75},
    ])
    assert len(ids) == 3
    assert odb.count("orders") == 3
    for id_ in ids:
        assert odb.get("orders", id_) is not None
    print(f"\n  â†’ 3 orders: {ids}")
    odb.close()

# â”€â”€ TEST 9 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_09_table_exists():
    p = db_path("t09_tables")
    odb = OverdriveDb.open(p)
    assert odb.table_exists("ghost") is False
    odb.create_table("real")
    assert odb.table_exists("real") is True
    print(f"\n  â†’ ghost=False, real=True")
    odb.close()

# â”€â”€ TEST 10 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
def test_10_version():
    v = OverdriveDb.version()
    assert v and len(v) > 0, "âŒ version empty"
    assert v != "unknown",   "âŒ native lib not loaded"
    assert v == "2.2.0",     f"âŒ expected 2.2.0, got {v}"
    print(f"\n  â†’ version: {v}")

