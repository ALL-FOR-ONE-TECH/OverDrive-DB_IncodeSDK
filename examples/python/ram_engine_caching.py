"""
OverDrive-DB — RAM Engine Caching Example (Python)

Demonstrates: full RAM database, per-table RAM storage, snapshot/restore,
memory usage monitoring, and rate limiting patterns.

Install:
    pip install overdrive-db

Run:
    python ram_engine_caching.py
"""
import time
from overdrive import OverDrive

def main():
    # ── Full RAM Database ───────────────────────────────────────────────────
    # engine="RAM" stores everything in memory — sub-microsecond reads.
    # The .odb path is used for snapshots; no disk I/O during normal operation.
    print("=== Full RAM Database ===")
    cache = OverDrive.open("cache.odb", engine="RAM")

    # Populate session cache
    for i in range(1, 101):
        cache.insert("sessions", {
            "user_id":  i,
            "token":    f"tok_{i:04d}",
            "ip":       f"10.0.0.{i % 255}",
            "expires":  "2026-12-31T23:59:59Z",
        })

    print(f"Cached {cache.count('sessions')} sessions")

    # Sub-microsecond read benchmark
    start = time.perf_counter()
    for _ in range(1000):
        cache.findOne("sessions", "user_id = 50")
    elapsed_ms = (time.perf_counter() - start) * 1000
    print(f"1000 reads in {elapsed_ms:.2f} ms ({elapsed_ms:.3f} µs avg)")

    # ── Memory Usage Monitoring ─────────────────────────────────────────────
    print("\n=== Memory Usage ===")
    usage = cache.memoryUsage()
    print(f"  Used:    {usage['bytes']:,} bytes ({usage['mb']:.3f} MB)")
    print(f"  Limit:   {usage['limit_bytes']:,} bytes")
    print(f"  Percent: {usage['percent']:.2f}%")

    # ── Snapshot — Persist RAM to Disk ─────────────────────────────────────
    print("\n=== Snapshot (RAM → Disk) ===")
    cache.snapshot("./cache_snapshot.odb")
    print("Snapshot saved to ./cache_snapshot.odb")

    # Simulate restart: close and re-open
    cache.close()
    print("Cache closed (data would be lost without snapshot)")

    # ── Restore — Load Snapshot Back into RAM ──────────────────────────────
    print("\n=== Restore (Disk → RAM) ===")
    cache2 = OverDrive.open("cache_restored.odb", engine="RAM")
    cache2.restore("./cache_snapshot.odb")
    print(f"Restored {cache2.count('sessions')} sessions from snapshot")

    # Verify data integrity
    session = cache2.findOne("sessions", "user_id = 50")
    print(f"Session 50 token: {session['token'] if session else 'not found'}")
    cache2.close()

    # ── Per-Table RAM Storage (Hybrid Mode) ────────────────────────────────
    # Mix RAM tables (hot data) with Disk tables (persistent data) in one DB.
    print("\n=== Hybrid: RAM Table + Disk Table ===")
    db = OverDrive.open("hybrid_app.odb")

    # Hot cache in RAM — fast reads for frequently accessed data
    db.create_table("rate_limits", engine="RAM")
    db.create_table("hot_sessions", engine="RAM")

    # Persistent data on disk
    # (auto-created on first insert — no createTable() needed)
    db.insert("users",  {"name": "Alice", "role": "admin"})
    db.insert("users",  {"name": "Bob",   "role": "user"})
    db.insert("orders", {"user": "Alice", "item": "widget", "qty": 3})

    # Populate RAM tables
    db.insert("rate_limits",  {"key": "user:1:api",  "count": 42, "window_sec": 60})
    db.insert("rate_limits",  {"key": "user:2:api",  "count": 7,  "window_sec": 60})
    db.insert("hot_sessions", {"token": "tok_abc", "user_id": 1, "ttl": 3600})

    print(f"  RAM  'rate_limits':  {db.count('rate_limits')} records")
    print(f"  RAM  'hot_sessions': {db.count('hot_sessions')} records")
    print(f"  Disk 'users':        {db.count('users')} records")
    print(f"  Disk 'orders':       {db.count('orders')} records")

    # ── Rate Limiting Pattern ───────────────────────────────────────────────
    print("\n=== Rate Limiting Pattern ===")

    def check_rate_limit(user_id: int, limit: int = 100) -> bool:
        """Return True if request is allowed, False if rate-limited."""
        key = f"user:{user_id}:api"
        record = db.findOne("rate_limits", f"key = '{key}'")
        if record is None:
            db.insert("rate_limits", {"key": key, "count": 1, "window_sec": 60})
            return True
        count = record.get("count", 0)
        if count >= limit:
            return False
        db.updateMany("rate_limits", f"key = '{key}'", {"count": count + 1})
        return True

    for user_id in [1, 2, 3]:
        allowed = check_rate_limit(user_id)
        print(f"  User {user_id}: {'allowed' if allowed else 'rate-limited'}")

    db.close()
    print("\nDone!")

if __name__ == "__main__":
    main()
