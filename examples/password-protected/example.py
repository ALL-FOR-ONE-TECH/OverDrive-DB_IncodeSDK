"""
OverDrive-DB — Password-Protected Database Example (Python)
"""
from overdrive import OverDrive

PASSWORD = "my-secret-password-123"

# Create encrypted database
print("Creating encrypted database...")
db = OverDrive.open("secure.odb", password=PASSWORD)
db.insert("secrets", {"key": "api_token", "value": "sk-abc123xyz"})
db.insert("secrets", {"key": "db_password", "value": "postgres-secret"})
print(f"Inserted {db.count('secrets')} secrets")
db.close()

# Re-open with correct password
print("\nRe-opening with correct password...")
db = OverDrive.open("secure.odb", password=PASSWORD)
secrets = db.findAll("secrets")
print(f"Retrieved {len(secrets)} secrets: {secrets}")
db.close()

# Try wrong password
print("\nTrying wrong password...")
try:
    db = OverDrive.open("secure.odb", password="wrong-password!!")
    print("ERROR: Should not reach here!")
except Exception as e:
    print(f"Correctly rejected: {e}")

# Try too short password
print("\nTrying short password...")
try:
    db = OverDrive.open("new.odb", password="short")
except Exception as e:
    print(f"Correctly rejected: {e}")

print("\nDone!")
