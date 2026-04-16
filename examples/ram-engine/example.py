"""
OverDrive-DB — RAM Engine Caching Example (Python)
"""
from overdrive import OverDrive

# --- Full RAM Database ---
print("=== Full RAM Database ===")
cache = OverDrive.open("cache.odb", engine="RAM")

# Insert session data (sub-microsecond reads)
for i in range(100):
    cache.insert("sessions", {
        "user_id": i,
        "token": f"token_{i}",
        "expires": "2026-12-31"
    })

print(f"Inserted {cache.count('sessions')} sessions")

# Check memory usage
usage = cache.memoryUsage()
print(f"RAM usage: {usage['mb']:.2f} MB ({usage['percent']:.1f}% of limit)")

# Snapshot to disk
cache.snapshot("./cache_snapshot.odb")
print("Snapshot saved to ./cache_snapshot.odb")

# Restore from snapshot
cache.restore("./cache_snapshot.odb")
print(f"Restored: {cache.count('sessions')} sessions")

cache.close()

# --- Per-Table RAM Storage ---
print("\n=== Per-Table RAM Storage ===")
db = OverDrive.open("hybrid.odb")

# Create a RAM table for hot data
db.createTable("hot_cache", engine="RAM")
db.insert("hot_cache", {"key": "rate_limit:user_1", "count": 42, "window": 60})

# Regular disk table for persistent data
db.insert("users", {"name": "Alice", "role": "admin"})

print(f"RAM table 'hot_cache': {db.count('hot_cache')} records")
print(f"Disk table 'users': {db.count('users')} records")

db.close()
print("\nDone!")
