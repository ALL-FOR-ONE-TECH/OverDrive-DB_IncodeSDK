"""
OverDrive-DB — Batch Operations Example (Python)
"""
from overdrive import OverDrive

db = OverDrive.open("batch_example.odb")

# --- Batch Insert ---
print("=== Batch Insert ===")
users = [{"name": f"User_{i}", "age": 20 + (i % 50), "status": "active"} for i in range(100)]
ids = db.insertMany("users", users)
print(f"Inserted {len(ids)} users")

# --- Batch Query ---
print("\n=== Batch Query ===")
young = db.findAll("users", "age < 30", order_by="age ASC")
print(f"Users under 30: {len(young)}")

old = db.findAll("users", "age >= 50", order_by="age DESC", limit=5)
print(f"Top 5 oldest: {[u['name'] for u in old]}")

# --- Batch Update ---
print("\n=== Batch Update ===")
count = db.updateMany("users", "age < 25", {"status": "junior"})
print(f"Updated {count} juniors")

count = db.updateMany("users", "age >= 50", {"status": "senior"})
print(f"Updated {count} seniors")

# --- Batch Count ---
print("\n=== Batch Count ===")
print(f"Total:   {db.countWhere('users')}")
print(f"Active:  {db.countWhere('users', 'status = \\'active\\'')}")
print(f"Junior:  {db.countWhere('users', 'status = \\'junior\\'')}")
print(f"Senior:  {db.countWhere('users', 'status = \\'senior\\'')}")

# --- Batch Delete ---
print("\n=== Batch Delete ===")
deleted = db.deleteMany("users", "status = 'junior'")
print(f"Deleted {deleted} juniors")
print(f"Remaining: {db.countWhere('users')}")

db.close()
print("\nDone!")
