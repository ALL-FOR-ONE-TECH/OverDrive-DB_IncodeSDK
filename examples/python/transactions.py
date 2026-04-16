"""
OverDrive-DB — Transactions Example (Python)

Demonstrates: callback pattern, manual pattern, retry helper, isolation levels.

Install:
    pip install overdrive-db

Run:
    python transactions.py
"""
from overdrive import OverDrive, TransactionError

def main():
    db = OverDrive.open("transactions.odb")

    # Seed some data
    db.insert("accounts", {"id": "acc_alice", "owner": "Alice", "balance": 1000})
    db.insert("accounts", {"id": "acc_bob",   "owner": "Bob",   "balance": 500})
    print("Initial balances:")
    for acc in db.findAll("accounts"):
        print(f"  {acc['owner']}: ${acc['balance']}")

    # ── Callback Pattern (v1.4 — recommended) ──────────────────────────────
    # The callback receives the transaction ID.
    # On success → auto-commit. On exception → auto-rollback.
    print("\n=== Callback Transaction ===")

    def transfer_funds(txn):
        db.updateMany("accounts", "id = 'acc_alice'", {"balance": 900})
        db.updateMany("accounts", "id = 'acc_bob'",   {"balance": 600})
        return "transferred $100 from Alice to Bob"

    result = db.transaction(transfer_funds)
    print(f"Result: {result}")

    # ── Callback with return value ──────────────────────────────────────────
    print("\n=== Transaction with Return Value ===")

    def create_order(txn):
        order_id = db.insert("orders", {"item": "widget", "qty": 3, "status": "pending"})
        db.insert("audit_log", {"event": "order_created", "order_id": order_id})
        return order_id

    order_id = db.transaction(create_order)
    print(f"Created order: {order_id}")

    # ── Rollback on Exception ───────────────────────────────────────────────
    print("\n=== Automatic Rollback on Error ===")
    count_before = db.count("orders")

    try:
        def failing_operation(txn):
            db.insert("orders", {"item": "gadget", "qty": 1})
            raise ValueError("Something went wrong — transaction will be rolled back")

        db.transaction(failing_operation)
    except ValueError as e:
        print(f"Caught expected error: {e}")

    count_after = db.count("orders")
    print(f"Orders before: {count_before}, after: {count_after} (unchanged — rolled back)")

    # ── Retry Helper ────────────────────────────────────────────────────────
    # transaction_with_retry() retries on TransactionError (e.g. deadlock)
    # with exponential backoff. Other exceptions propagate immediately.
    print("\n=== Transaction with Retry ===")

    def safe_insert(txn):
        db.insert("events", {"type": "page_view", "url": "/home"})
        return "event logged"

    result = db.transaction_with_retry(safe_insert, max_retries=3)
    print(f"Retry result: {result}")

    # ── Isolation Levels ────────────────────────────────────────────────────
    print("\n=== Isolation Levels ===")

    # READ_COMMITTED (default) — no dirty reads
    def read_committed_txn(txn):
        return db.findAll("accounts")

    accounts = db.transaction(read_committed_txn, isolation=OverDrive.READ_COMMITTED)
    print(f"READ_COMMITTED: {len(accounts)} accounts")

    # SERIALIZABLE — full serial isolation
    def serializable_txn(txn):
        count = db.countWhere("accounts", "balance > 0")
        db.insert("snapshots", {"account_count": count})
        return count

    count = db.transaction(serializable_txn, isolation=OverDrive.SERIALIZABLE)
    print(f"SERIALIZABLE: {count} accounts with positive balance")

    # ── Manual Pattern (v1.3 — still fully supported) ───────────────────────
    print("\n=== Manual Transaction (v1.3 style) ===")
    txn_id = db.begin_transaction()
    try:
        db.insert("logs", {"event": "manual_txn_test", "status": "started"})
        db.commit_transaction(txn_id)
        print("Manual transaction committed")
    except Exception as e:
        db.abort_transaction(txn_id)
        print(f"Manual transaction rolled back: {e}")

    # ── Context Manager Pattern ─────────────────────────────────────────────
    print("\n=== Context Manager Pattern ===")
    with db.transaction() as txn_id:
        db.insert("logs", {"event": "context_manager_txn", "txn_id": txn_id})
    print("Context manager transaction committed")

    # Final state
    print("\n=== Final Balances ===")
    for acc in db.findAll("accounts"):
        print(f"  {acc['owner']}: ${acc['balance']}")

    db.close()
    print("\nDone!")

if __name__ == "__main__":
    main()
