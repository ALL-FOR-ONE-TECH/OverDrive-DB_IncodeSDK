"""
OverDrive-DB — Transaction Examples (Python)
"""
from overdrive import OverDrive

db = OverDrive.open("txn_example.odb")

# Setup
db.insert("accounts", {"id": "acc_1", "name": "Alice", "balance": 1000})
db.insert("accounts", {"id": "acc_2", "name": "Bob", "balance": 1000})

# --- Callback Pattern (v1.4) ---
def transfer(txn):
    db.updateMany("accounts", "id = 'acc_1'", {"balance": 900})
    db.updateMany("accounts", "id = 'acc_2'", {"balance": 1100})
    return "transferred $100"

result = db.transaction(transfer)
print(f"Transaction result: {result}")

# --- With Retry ---
def risky_operation(txn):
    db.insert("logs", {"event": "audit", "ts": "2026-04-15"})
    return "logged"

result = db.transaction_with_retry(risky_operation, max_retries=3)
print(f"Retry result: {result}")

# --- Manual Pattern (v1.3, still works) ---
txn_id = db.begin_transaction()
try:
    db.insert("logs", {"event": "manual_txn"})
    db.commit_transaction(txn_id)
    print("Manual transaction committed")
except:
    db.abort_transaction(txn_id)
    print("Manual transaction rolled back")

# Verify
print("Accounts:", db.findAll("accounts"))
print("Logs:", db.findAll("logs"))

db.close()
