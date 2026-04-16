"""
OverDrive-DB — Watchdog Monitoring Example (Python)
"""
from overdrive import OverDrive
import time

# Create a database first
db = OverDrive.open("monitored.odb")
db.insert("data", {"key": "value"})
db.close()

# --- Monitor database integrity ---
print("=== Watchdog Report ===")
report = OverDrive.watchdog("monitored.odb")

print(f"  File:      {report.file_path}")
print(f"  Size:      {report.file_size_bytes:,} bytes")
print(f"  Modified:  {report.last_modified}")
print(f"  Status:    {report.integrity_status}")
print(f"  Pages:     {report.page_count}")
print(f"  Magic OK:  {report.magic_valid}")

if report.integrity_status == "valid":
    print("\n✓ Database is healthy!")
elif report.integrity_status == "corrupted":
    print(f"\n⚠ CORRUPTED: {report.corruption_details}")
elif report.integrity_status == "missing":
    print("\n⚠ Database file not found!")

# --- Monitor missing file ---
print("\n=== Missing File Test ===")
report2 = OverDrive.watchdog("nonexistent.odb")
print(f"  Status: {report2.integrity_status}")

# --- Periodic monitoring ---
print("\n=== Periodic Monitoring ===")
for i in range(3):
    report = OverDrive.watchdog("monitored.odb")
    status = "✓" if report.integrity_status == "valid" else "✗"
    print(f"  Check {i+1}: {status} ({report.file_size_bytes:,} bytes)")
    time.sleep(1)

print("\nDone!")
