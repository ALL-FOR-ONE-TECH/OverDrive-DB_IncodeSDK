"""
OverDrive-DB — Basic CRUD Example (Python)
"""
from overdrive import OverDrive

def main():
    # Open database (auto-creates tables)
    db = OverDrive.open("example.odb")

    # INSERT
    id1 = db.insert("users", {"name": "Alice", "age": 30, "email": "alice@example.com"})
    id2 = db.insert("users", {"name": "Bob", "age": 25, "email": "bob@example.com"})
    id3 = db.insert("users", {"name": "Charlie", "age": 35, "email": "charlie@example.com"})
    print(f"Inserted: {id1}, {id2}, {id3}")

    # GET
    user = db.get("users", id1)
    print(f"Got user: {user}")

    # UPDATE
    db.update("users", id1, {"age": 31})
    print(f"Updated Alice's age to 31")

    # QUERY
    results = db.query("SELECT * FROM users WHERE age > 25 ORDER BY name")
    print(f"Users over 25: {results}")

    # HELPER METHODS
    first = db.findOne("users", "name = 'Alice'")
    print(f"findOne: {first}")

    all_users = db.findAll("users", order_by="age DESC", limit=10)
    print(f"findAll: {all_users}")

    count = db.countWhere("users", "age > 25")
    print(f"Count age > 25: {count}")

    exists = db.exists("users", id1)
    print(f"User {id1} exists: {exists}")

    # DELETE
    db.delete("users", id3)
    print(f"Deleted {id3}")

    # BULK OPERATIONS
    updated = db.updateMany("users", "age < 30", {"status": "young"})
    print(f"Updated {updated} users")

    deleted = db.deleteMany("users", "age > 100")
    print(f"Deleted {deleted} old users")

    db.close()
    print("Done!")

if __name__ == "__main__":
    main()
