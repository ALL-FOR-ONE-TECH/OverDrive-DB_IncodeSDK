"""
OverDrive-DB — Watchdog Monitoring Example (Python)

Demonstrates: integrity checks, health monitoring, pre-open validation,
periodic monitoring, and backup verification.

Install:
    pip install overdrive-db

Run:
    python watchdog_monitoring.py
"""
import time
from overdrive import OverDrive

def print_report(report):
    """Pretty-print a WatchdogReport."""
    status_icon = {"valid": "✓", "corrupted": "✗", "missing": "?"}.get(
        report.integrity_status, "?"
    )
    print(f"  {status_icon} Status:   {report.integrity_status}")
    print(f"    Path:     {report.file_path}")
    print(f"    Size:     {report.file_size_bytes:,} bytes")
    print(f"    Modified: {report.last_modified}")
    print(f"    Pages:    {report.page_count}")
    print(f"    Magic OK: {report.magic_valid}")
    if report.corruption_details:
        print(f"    Details:  {report.corruption_details}")

def main():
    # ── Create a database to monitor ───────────────────────────────────────
    db = OverDrive.open("monitored.odb")
    for i in range(10):
        db.insert("metrics", {"sensor": f"sensor_{i}", "value": i * 1.5, "unit": "°C"})
    db.close()

    # ── Basic Integrity Check ───────────────────────────────────────────────
    # watchdog() is a static method — no open database handle required.
    # Completes in < 100ms for files under 1 GB.
    print("=== Basic Integrity Check ===")
    report = OverDrive.watchdog("monitored.odb")
    print_report(report)

    if report.integrity_status == "valid":
        print("  → Safe to open")
        db = OverDrive.open("monitored.odb")
        print(f"  → Opened successfully, {db.count('metrics')} records")
        db.close()

    # ── Missing File ────────────────────────────────────────────────────────
    print("\n=== Missing File ===")
    report = OverDrive.watchdog("nonexistent.odb")
    print_report(report)

    # ── Pre-Open Validation Pattern ─────────────────────────────────────────
    # Always check integrity before opening in production.
    print("\n=== Pre-Open Validation Pattern ===")

    def safe_open(path: str, **kwargs):
        """Open a database only after passing a watchdog check."""
        report = OverDrive.watchdog(path)
        if report.integrity_status == "missing":
            # New database — safe to create
            print(f"  Creating new database: {path}")
            return OverDrive.open(path, **kwargs)
        elif report.integrity_status == "corrupted":
            raise RuntimeError(
                f"Database {path} is corrupted: {report.corruption_details}\n"
                "Restore from backup before opening."
            )
        else:
            print(f"  Opening healthy database: {path} ({report.file_size_bytes:,} bytes)")
            return OverDrive.open(path, **kwargs)

    db = safe_open("monitored.odb")
    db.close()

    # ── Backup Verification ─────────────────────────────────────────────────
    print("\n=== Backup Verification ===")
    # Create a backup
    db = OverDrive.open("monitored.odb")
    db.backup("monitored_backup.odb")
    db.close()

    # Verify the backup is intact before trusting it
    backup_report = OverDrive.watchdog("monitored_backup.odb")
    print_report(backup_report)
    if backup_report.integrity_status == "valid":
        print("  → Backup verified — safe to use for restore")
    else:
        print("  → Backup is invalid — do not use for restore!")

    # ── Periodic Monitoring ─────────────────────────────────────────────────
    # Run watchdog on a schedule to detect corruption early.
    print("\n=== Periodic Monitoring (3 checks, 1s apart) ===")
    for i in range(1, 4):
        report = OverDrive.watchdog("monitored.odb")
        icon = "✓" if report.integrity_status == "valid" else "✗"
        print(f"  Check {i}: {icon} {report.integrity_status} "
              f"({report.file_size_bytes:,} bytes, {report.page_count} pages)")
        if report.integrity_status != "valid":
            print(f"    ALERT: {report.corruption_details}")
        time.sleep(1)

    # ── Monitor Multiple Databases ──────────────────────────────────────────
    print("\n=== Monitor Multiple Databases ===")
    databases = ["monitored.odb", "monitored_backup.odb", "nonexistent.odb"]
    for db_path in databases:
        report = OverDrive.watchdog(db_path)
        icon = {"valid": "✓", "corrupted": "✗", "missing": "?"}.get(
            report.integrity_status, "?"
        )
        print(f"  {icon} {db_path}: {report.integrity_status}")

    print("\nDone!")

if __name__ == "__main__":
    main()
