"""
OverDrive-DB — Password-Protected Database Example (Python)

Demonstrates: creating encrypted databases, opening with correct/wrong passwords,
password validation, and environment variable best practices.

Install:
    pip install overdrive-db

Run:
    python password_protected.py
"""
import os
from overdrive import OverDrive, AuthenticationError, OverDriveError

# ── Best practice: load password from environment variable ──────────────────
# Never hardcode passwords in source code.
# Set with: $env:DB_PASSWORD = "my-secret-password-123"  (PowerShell)
#       or: export DB_PASSWORD="my-secret-password-123"  (bash)
PASSWORD = os.environ.get("DB_PASSWORD", "my-secret-password-123")

def main():
    # ── Create an encrypted database ───────────────────────────────────────
    print("=== Creating Encrypted Database ===")
    # The password is used to derive an AES-256-GCM key via Argon2id.
    # The salt is stored in the file header — you only need the password to re-open.
    db = OverDrive.open("secure.odb", password=PASSWORD)

    db.insert("secrets", {"key": "api_token",    "value": "sk-abc123xyz"})
    db.insert("secrets", {"key": "db_password",  "value": "postgres-secret"})
    db.insert("secrets", {"key": "webhook_secret","value": "whsec-xyz789"})

    count = db.count("secrets")
    print(f"Stored {count} secrets in encrypted database")
    db.close()
    print("Database closed.")

    # ── Re-open with the correct password ──────────────────────────────────
    print("\n=== Re-opening with Correct Password ===")
    db = OverDrive.open("secure.odb", password=PASSWORD)
    secrets = db.findAll("secrets")
    print(f"Retrieved {len(secrets)} secrets:")
    for s in secrets:
        # Show key names but mask values in output
        print(f"  {s['key']}: {'*' * len(str(s['value']))}")
    db.close()

    # ── Wrong password is rejected ──────────────────────────────────────────
    print("\n=== Wrong Password (Expected Rejection) ===")
    try:
        db = OverDrive.open("secure.odb", password="wrong-password-xyz!")
        print("ERROR: Should not reach here!")
        db.close()
    except AuthenticationError as e:
        print(f"Correctly rejected with AuthenticationError: {e.code}")
    except OverDriveError as e:
        print(f"Correctly rejected: {e}")

    # ── Password too short is rejected ─────────────────────────────────────
    print("\n=== Short Password Validation ===")
    try:
        db = OverDrive.open("short_pass.odb", password="short")
        print("ERROR: Should not reach here!")
        db.close()
    except OverDriveError as e:
        print(f"Correctly rejected (< 8 chars): {e}")

    # ── Password + engine combination ──────────────────────────────────────
    print("\n=== Encrypted RAM Database ===")
    # You can combine password protection with any storage engine
    ram_db = OverDrive.open("secure_cache.odb", password=PASSWORD, engine="RAM")
    ram_db.insert("sessions", {"user_id": 42, "token": "tok_abc", "expires": "2026-12-31"})
    print(f"Encrypted RAM sessions: {ram_db.count('sessions')}")
    ram_db.snapshot("secure_cache_snapshot.odb")
    print("Snapshot saved.")
    ram_db.close()

    # ── Disable auto-table creation for strict mode ─────────────────────────
    print("\n=== Strict Mode (auto_create_tables=False) ===")
    strict_db = OverDrive.open("strict.odb", password=PASSWORD, auto_create_tables=False)
    strict_db.create_table("allowed_table")
    strict_db.insert("allowed_table", {"data": "ok"})
    print("Insert into pre-created table: OK")

    try:
        strict_db.insert("unknown_table", {"data": "should fail"})
        print("ERROR: Should not reach here!")
    except OverDriveError as e:
        print(f"Correctly rejected insert into unknown table: {type(e).__name__}")
    strict_db.close()

    print("\nDone!")

if __name__ == "__main__":
    main()
