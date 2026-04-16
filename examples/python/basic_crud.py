"""
OverDrive-DB — Basic CRUD Example (Python)

Demonstrates: insert, get, update, delete, query, and helper methods.

Install:
    pip install overdrive-db

Run:
    python basic_crud.py
"""
from overdrive import OverDrive

def main():
    # Open (or create) a database.
    # Tables are auto-created on first insert — no createTable() needed.
    db = OverDrive.open("basic_crud.odb")

    print("=== INSERT ===")
    # Insert documents — each gets an auto-generated _id like "users_1"
    id1 = db.insert("users", {"name": "Alice", "age": 30, "email": "alice@example.com"})
    id2 = db.insert("users", {"name": "Bob",   "age": 25, "email": "bob@example.com"})
    id3 = db.insert("users", {"name": "Charlie","age": 35, "email": "charlie@example.com"})
    print(f"Inserted: {id1}, {id2}, {id3}")

    print("\n=== GET ===")
    # Retrieve a single document by its _id
    user = db.get("users", id1)
    print(f"Got: {user}")

    print("\n=== UPDATE ===")
    # Update specific fields on a document
    db.update("users", id1, {"age": 31, "status": "active"})
    print(f"Updated Alice: {db.get('users', id1)}")

    print("\n=== DELETE ===")
    # Delete a document by _id
    deleted = db.delete("users", id3)
    print(f"Deleted {id3}: {deleted}")
    print(f"Remaining count: {db.count('users')}")

    print("\n=== SQL QUERY ===")
    # Execute SQL directly — full SELECT/WHERE/ORDER BY support
    results = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name")
    for row in results:
        print(f"  {row['name']} (age {row['age']})")

    print("\n=== HELPER METHODS ===")
    # findOne — returns first match or None
    alice = db.findOne("users", "name = 'Alice'")
    print(f"findOne: {alice['name'] if alice else 'not found'}")

    # findAll — returns all matches with optional sorting and limit
    all_users = db.findAll("users", order_by="age DESC", limit=10)
    print(f"findAll ({len(all_users)} users): {[u['name'] for u in all_users]}")

    # findAll with WHERE clause
    adults = db.findAll("users", where="age >= 30")
    print(f"Adults: {[u['name'] for u in adults]}")

    # countWhere — count matching documents
    count = db.countWhere("users", "age > 25")
    print(f"countWhere age > 25: {count}")

    # exists — check if a document exists by _id
    print(f"exists({id1}): {db.exists('users', id1)}")
    print(f"exists('users_999'): {db.exists('users', 'users_999')}")

    print("\n=== BULK OPERATIONS ===")
    # updateMany — update all matching documents, returns count
    updated = db.updateMany("users", "age < 30", {"tier": "standard"})
    print(f"updateMany: updated {updated} users")

    # deleteMany — delete all matching documents, returns count
    removed = db.deleteMany("users", "tier = 'standard'")
    print(f"deleteMany: removed {removed} users")

    print(f"\nFinal count: {db.count('users')}")

    db.close()
    print("\nDone!")

if __name__ == "__main__":
    main()
