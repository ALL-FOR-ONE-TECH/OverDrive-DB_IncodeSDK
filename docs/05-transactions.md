# Transactions

OverDrive uses MVCC (Multi-Version Concurrency Control) for transactions — the same approach used by PostgreSQL. This gives you strong consistency without sacrificing performance.

---

## Why Use Transactions?

Transactions ensure that a group of operations either **all succeed** or **all fail together**. Without transactions, a crash or error mid-way through could leave your data in an inconsistent state.

**Example — bank transfer:**
```python
# Without transaction — DANGEROUS
db.updateMany("accounts", "id = 'alice'", {"balance": 900})
# ← if the app crashes here, Alice loses $100 but Bob never gets it
db.updateMany("accounts", "id = 'bob'",   {"balance": 600})

# With transaction — SAFE
def transfer(txn):
    db.updateMany("accounts", "id = 'alice'", {"balance": 900})
    db.updateMany("accounts", "id = 'bob'",   {"balance": 600})

db.transaction(transfer)  # either both happen, or neither does
```

---

## The Callback Pattern (Recommended — v1.4)

The simplest way to use transactions. Pass a function — it auto-commits on success and auto-rolls back on any exception.

### Python

```python
from overdrive import OverDrive

db = OverDrive.open("bank.odb")
db.insert("accounts", {"id": "alice", "balance": 1000})
db.insert("accounts", {"id": "bob",   "balance": 500})

# Simple callback
def transfer_100(txn):
    db.updateMany("accounts", "id = 'alice'", {"balance": 900})
    db.updateMany("accounts", "id = 'bob'",   {"balance": 600})
    return "transfer complete"

result = db.transaction(transfer_100)
print(result)  # "transfer complete"

# Lambda for simple cases
result = db.transaction(lambda txn: db.insert("logs", {"event": "transfer"}))

# With isolation level
result = db.transaction(transfer_100, isolation=OverDrive.SERIALIZABLE)
```

### Node.js

```javascript
const { OverDrive } = require('overdrive-db');
const db = OverDrive.open('bank.odb');

// Sync callback
const result = db.transaction((txn) => {
    db.updateMany('accounts', "id = 'alice'", { balance: 900 });
    db.updateMany('accounts', "id = 'bob'",   { balance: 600 });
    return 'transfer complete';
});
console.log(result);  // "transfer complete"

// Async callback — returns a Promise
const asyncResult = await db.transaction(async (txn) => {
    const orderId = db.insert('orders', { item: 'widget', qty: 1 });
    await someExternalCall();  // async work inside transaction
    return orderId;
});
```

### Java

```java
try (OverDrive db = OverDrive.open("bank.odb")) {
    // Generic callback — returns any type
    String result = db.transaction(txn -> {
        db.updateMany("accounts", "id = 'alice'", Map.of("balance", 900));
        db.updateMany("accounts", "id = 'bob'",   Map.of("balance", 600));
        return "transfer complete";
    });
    System.out.println(result);

    // With isolation level
    Integer count = db.transaction(txn -> {
        db.insert("logs", Map.of("event", "transfer"));
        return db.count("logs");
    }, OverDrive.SERIALIZABLE);
}
```

### Go

```go
db, _ := overdrive.Open("bank.odb")
defer db.Close()

err := db.Transaction(func(txn *overdrive.TransactionHandle) error {
    _, err := db.UpdateMany("accounts", "id = 'alice'", map[string]any{"balance": 900})
    if err != nil { return err }
    _, err = db.UpdateMany("accounts", "id = 'bob'", map[string]any{"balance": 600})
    return err
}, overdrive.ReadCommitted)

if err != nil {
    fmt.Println("Transaction failed:", err)
}
```

---

## Automatic Rollback on Error

If your callback throws an exception, the transaction is automatically rolled back:

```python
db = OverDrive.open("app.odb")
db.insert("accounts", {"id": "alice", "balance": 1000})

count_before = db.count("accounts")

try:
    def bad_operation(txn):
        db.insert("accounts", {"id": "temp", "balance": 0})
        raise ValueError("Something went wrong!")  # ← triggers rollback

    db.transaction(bad_operation)
except ValueError:
    pass

count_after = db.count("accounts")
print(count_before == count_after)  # True — the insert was rolled back
```

---

## Transaction with Retry

For high-concurrency scenarios, transactions can fail due to conflicts. Use `transactionWithRetry()` to automatically retry with exponential backoff:

```python
# Retries up to 3 times on TransactionError (deadlock/conflict)
# Other exceptions propagate immediately without retrying
result = db.transaction_with_retry(
    lambda txn: db.insert("orders", {"item": "widget"}),
    max_retries=3
)
```

```javascript
// Node.js
const result = await db.transactionWithRetry(
    async (txn) => db.insert('orders', { item: 'widget' }),
    OverDrive.READ_COMMITTED,
    3  // max retries
);
```

```java
// Java
String result = db.transactionWithRetry(txn -> {
    db.insert("orders", Map.of("item", "widget"));
    return "done";
}, OverDrive.READ_COMMITTED, 3);
```

```go
// Go
err := db.TransactionWithRetry(func(txn *overdrive.TransactionHandle) error {
    _, err := db.Insert("orders", map[string]any{"item": "widget"})
    return err
}, overdrive.ReadCommitted, 3)
```

---

## Manual Transaction Pattern (v1.3 — Still Supported)

If you need more control, use the manual begin/commit/abort pattern:

```python
# Python — manual pattern
txn_id = db.begin_transaction()
try:
    db.insert("users", {"name": "Alice"})
    db.insert("logs", {"event": "user_created"})
    db.commit_transaction(txn_id)
    print("Committed!")
except Exception as e:
    db.abort_transaction(txn_id)
    print(f"Rolled back: {e}")
```

```python
# Python — context manager pattern (also v1.3)
with db.transaction() as txn_id:
    db.insert("users", {"name": "Alice"})
    # auto-commits on exit, auto-aborts on exception
```

```javascript
// Node.js — manual pattern
const txnId = db.beginTransaction();
try {
    db.insert('users', { name: 'Alice' });
    db.commitTransaction(txnId);
} catch (e) {
    db.abortTransaction(txnId);
}

// Node.js — context object pattern
const txn = db.transaction();  // no callback = returns context object
txn.commit();   // or txn.abort()
```

```java
// Java — manual pattern
long txnId = db.beginTransaction();
try {
    db.insert("users", Map.of("name", "Alice"));
    db.commitTransaction(txnId);
} catch (Exception e) {
    db.abortTransaction(txnId);
}
```

```go
// Go — manual pattern
txn, err := db.BeginTransaction(overdrive.ReadCommitted)
if err != nil { panic(err) }

_, err = db.Insert("users", map[string]any{"name": "Alice"})
if err != nil {
    db.AbortTransaction(txn)
    return
}
db.CommitTransaction(txn)
```

---

## Isolation Levels

Choose how strictly transactions are isolated from each other:

| Level | Constant | Dirty Reads | Non-Repeatable Reads | Phantom Reads |
|-------|----------|-------------|---------------------|---------------|
| Read Uncommitted | `READ_UNCOMMITTED` | ✅ Possible | ✅ Possible | ✅ Possible |
| Read Committed | `READ_COMMITTED` | ❌ Prevented | ✅ Possible | ✅ Possible |
| Repeatable Read | `REPEATABLE_READ` | ❌ Prevented | ❌ Prevented | ✅ Possible |
| Serializable | `SERIALIZABLE` | ❌ Prevented | ❌ Prevented | ❌ Prevented |

**Default:** `READ_COMMITTED` — good for most applications.

```python
# Python
db.transaction(my_fn, isolation=OverDrive.READ_COMMITTED)   # default
db.transaction(my_fn, isolation=OverDrive.REPEATABLE_READ)
db.transaction(my_fn, isolation=OverDrive.SERIALIZABLE)

# Constants
OverDrive.READ_UNCOMMITTED  # 0
OverDrive.READ_COMMITTED    # 1
OverDrive.REPEATABLE_READ   # 2
OverDrive.SERIALIZABLE      # 3
```

**When to use each:**
- `READ_COMMITTED` — most web apps, APIs, general CRUD
- `REPEATABLE_READ` — reports, analytics that read the same data multiple times
- `SERIALIZABLE` — financial transactions, inventory management, anything requiring strict consistency

---

## Practical Example — Order Processing

```python
from overdrive import OverDrive, TransactionError

db = OverDrive.open("store.odb")

# Setup
db.insert("inventory", {"product_id": "laptop", "stock": 5, "price": 999})
db.insert("inventory", {"product_id": "mouse",  "stock": 50, "price": 29})

def place_order(user_id: str, product_id: str, qty: int):
    """Place an order — atomically checks stock and creates order."""

    def order_txn(txn):
        # Check stock
        item = db.findOne("inventory", f"product_id = '{product_id}'")
        if item is None:
            raise ValueError(f"Product {product_id} not found")
        if item["stock"] < qty:
            raise ValueError(f"Insufficient stock: {item['stock']} available, {qty} requested")

        # Deduct stock
        new_stock = item["stock"] - qty
        db.updateMany("inventory", f"product_id = '{product_id}'", {"stock": new_stock})

        # Create order
        order_id = db.insert("orders", {
            "user_id": user_id,
            "product_id": product_id,
            "qty": qty,
            "total": item["price"] * qty,
            "status": "confirmed"
        })

        return order_id

    # Retry up to 3 times on conflict
    return db.transaction_with_retry(order_txn, max_retries=3)

# Place orders
order1 = place_order("user_1", "laptop", 2)
print(f"Order 1: {order1}")

order2 = place_order("user_2", "mouse", 10)
print(f"Order 2: {order2}")

try:
    order3 = place_order("user_3", "laptop", 10)  # only 3 left
except ValueError as e:
    print(f"Order 3 failed: {e}")  # "Insufficient stock: 3 available, 10 requested"

# Check inventory
for item in db.findAll("inventory"):
    print(f"  {item['product_id']}: {item['stock']} remaining")

db.close()
```
