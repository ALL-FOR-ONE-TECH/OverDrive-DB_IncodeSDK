"""
OverDrive-DB Python SDK — Sample Code (v1.4.3)
Install: pip install overdrive-db
"""
from overdrive import OverDrive

# ── 1. Open database ──────────────────────────
db = OverDrive.open("myapp.odb")

# ── 2. Insert documents (table auto-created) ──
db.insert("users", {"name": "Alice", "age": 30, "role": "admin"})
db.insert("users", {"name": "Bob",   "age": 25, "role": "user"})
db.insert("users", {"name": "Carol", "age": 35, "role": "admin"})

# ── 3. Query with SQL ─────────────────────────
results = db.query("SELECT * FROM users WHERE age > 28")
print(f"Users over 28: {len(results)}")
for r in results:
    print(f"  {r['name']} (age {r['age']})")

# ── 4. Helper methods ─────────────────────────
alice = db.findOne("users", "name = 'Alice'")
print(f"\nFound: {alice['name']}, role={alice['role']}")

admins = db.findAll("users", where="role = 'admin'")
print(f"Admins: {len(admins)}")

count = db.countWhere("users", "age > 25")
print(f"Users older than 25: {count}")

# ── 5. Update & delete ────────────────────────
updated = db.updateMany("users", "role = 'user'", {"verified": True})
print(f"\nVerified {updated} user(s)")

# ── 6. Transactions ───────────────────────────
result = db.transaction(lambda txn:
    db.insert("users", {"name": "Dave", "age": 28, "role": "user"})
)
print(f"Transaction committed, new ID: {result}")

# ── 7. Parameterized queries (safe from injection) ──
safe = db.querySafe(
    "SELECT * FROM users WHERE name = ?",
    ["Alice"]
)
print(f"\nSafe query found: {len(safe)} result(s)")

# ── 8. Cleanup ────────────────────────────────
db.close()
print("\n✅ Done!")
