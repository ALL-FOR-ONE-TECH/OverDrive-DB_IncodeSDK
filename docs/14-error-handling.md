# Error Handling Guide

OverDrive-DB uses structured errors with machine-readable codes, human-readable messages, and actionable suggestions.

---

## Error Code Format

```
ODB-{CATEGORY}-{NUMBER}

Examples:
  ODB-AUTH-001   ← Authentication error #1
  ODB-TABLE-001  ← Table error #1
  ODB-QUERY-001  ← Query error #1
```

### Categories

| Category | Prefix | What It Covers |
|----------|--------|----------------|
| Authentication | `ODB-AUTH-` | Wrong password, password too short |
| Table | `ODB-TABLE-` | Table not found, already exists |
| Query | `ODB-QUERY-` | SQL syntax errors |
| Transaction | `ODB-TXN-` | Deadlocks, conflicts |
| I/O | `ODB-IO-` | File not found, permission denied, corruption |
| FFI | `ODB-FFI-` | Native library not found, null handle |

---

## Error Hierarchy

```
OverDriveError (base)
├── AuthenticationError   ODB-AUTH-*
├── TableError            ODB-TABLE-*
├── QueryError            ODB-QUERY-*
├── TransactionError      ODB-TXN-*
├── IOError               ODB-IO-*
└── FFIError              ODB-FFI-*
```

Every error has these properties:
- `code` — machine-readable code (e.g. `ODB-AUTH-001`)
- `message` — human-readable description
- `context` — additional context (e.g. the database path or table name)
- `suggestions` — list of actionable steps to fix the problem
- `docLink` — URL to detailed documentation

---

## Python Error Handling

```python
from overdrive import (
    OverDrive,
    OverDriveError,
    AuthenticationError,
    TableError,
    QueryError,
    TransactionError,
    OverDriveIOError,
    FFIError,
)

# Catch specific error types
try:
    db = OverDrive.open("secure.odb", password="wrong")
except AuthenticationError as e:
    print(f"Code: {e.code}")           # ODB-AUTH-001
    print(f"Message: {e.message}")     # Incorrect password for database 'secure.odb'
    print(f"Context: {e.context}")     # secure.odb
    print(f"Suggestions: {e.suggestions}")
    print(f"Docs: {e.doc_link}")

# Catch table errors
try:
    db.query("SELECT * FROM nonexistent_table")
except TableError as e:
    print(f"Table error: {e.code} — {e.message}")

# Catch query errors
try:
    db.query("INVALID SQL !!!")
except QueryError as e:
    print(f"SQL error: {e.message}")

# Catch transaction conflicts
try:
    db.transaction(lambda txn: db.insert("orders", {"item": "widget"}))
except TransactionError as e:
    print(f"Transaction conflict: {e.code}")
    # Use transaction_with_retry() to handle this automatically

# Catch all OverDrive errors
try:
    db.insert("users", {"name": "Alice"})
except OverDriveError as e:
    print(f"[{e.code}] {e.message}")
    if e.suggestions:
        print("Suggestions:")
        for s in e.suggestions:
            print(f"  • {s}")
```

---

## Node.js Error Handling

```javascript
const {
    OverDrive,
    OverDriveError,
    AuthenticationError,
    TableError,
    QueryError,
    TransactionError,
    OverDriveIOError,
    FFIError,
} = require('overdrive-db');

// Catch specific error types
try {
    const db = OverDrive.open('secure.odb', { password: 'wrong' });
} catch (e) {
    if (e instanceof AuthenticationError) {
        console.log(`Code: ${e.code}`);           // ODB-AUTH-001
        console.log(`Message: ${e.message}`);
        console.log(`Context: ${e.context}`);
        console.log(`Suggestions: ${e.suggestions}`);
        console.log(`Docs: ${e.docLink}`);
    }
}

// Catch all OverDrive errors
try {
    db.query('INVALID SQL !!!');
} catch (e) {
    if (e instanceof OverDriveError) {
        console.log(`[${e.code}] ${e.message}`);
        if (e.suggestions?.length) {
            console.log('Suggestions:');
            e.suggestions.forEach(s => console.log(`  • ${s}`));
        }
    }
}

// Async error handling
async function safeInsert(db, table, doc) {
    try {
        return await db.transaction(async (txn) => {
            return db.insert(table, doc);
        });
    } catch (e) {
        if (e instanceof TransactionError) {
            // Retry with transactionWithRetry()
            return db.transactionWithRetry(
                async (txn) => db.insert(table, doc),
                OverDrive.READ_COMMITTED,
                3
            );
        }
        throw e;
    }
}
```

---

## Java Error Handling

```java
import com.afot.overdrive.OverDriveException;
import com.afot.overdrive.OverDriveException.*;

// Catch specific exception types
try {
    OverDrive db = OverDrive.open("secure.odb",
        new OverDrive.OpenOptions().password("wrong"));
} catch (AuthenticationException e) {
    System.out.println("Code: " + e.getCode());           // ODB-AUTH-001
    System.out.println("Message: " + e.getMessage());
    System.out.println("Context: " + e.getContext());
    System.out.println("Suggestions: " + e.getSuggestions());
    System.out.println("Docs: " + e.getDocLink());
} catch (FFIException e) {
    System.out.println("Native library error: " + e.getMessage());
    System.out.println("Install instructions: " + e.getSuggestions());
}

// Catch all OverDrive exceptions
try {
    db.query("INVALID SQL !!!");
} catch (OverDriveException e) {
    System.out.printf("[%s] %s%n", e.getCode(), e.getMessage());
    if (!e.getSuggestions().isEmpty()) {
        System.out.println("Suggestions:");
        e.getSuggestions().forEach(s -> System.out.println("  • " + s));
    }
}

// Transaction retry pattern
try {
    String result = db.transactionWithRetry(txn -> {
        db.insert("orders", Map.of("item", "widget"));
        return "done";
    }, OverDrive.READ_COMMITTED, 3);
} catch (TransactionException e) {
    System.out.println("All retries failed: " + e.getMessage());
}
```

---

## Go Error Handling

```go
import overdrive "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"

// Check error type
db, err := overdrive.Open("secure.odb", overdrive.WithPassword("wrong"))
if err != nil {
    switch e := err.(type) {
    case *overdrive.AuthenticationError:
        fmt.Printf("Auth error [%s]: %s\n", e.Code(), e.Error())
        fmt.Printf("Suggestions: %v\n", e.Suggestions())
    case *overdrive.FFIError:
        fmt.Printf("Library error [%s]: %s\n", e.Code(), e.Error())
    default:
        // Use the OverDriveError interface
        if ode, ok := err.(overdrive.OverDriveError); ok {
            fmt.Printf("[%s] %s\n", ode.Code(), ode.Error())
        } else {
            fmt.Printf("Unknown error: %v\n", err)
        }
    }
}

// Transaction retry
err = db.TransactionWithRetry(func(txn *overdrive.TransactionHandle) error {
    _, err := db.Insert("orders", map[string]any{"item": "widget"})
    return err
}, overdrive.ReadCommitted, 3)

if err != nil {
    if txnErr, ok := err.(*overdrive.TransactionError); ok {
        fmt.Printf("All retries failed: %s\n", txnErr.Error())
    }
}
```

---

## Common Errors and Solutions

### ODB-AUTH-001 — Incorrect Password

```
Error ODB-AUTH-001: Incorrect password for database 'app.odb'

Suggestions:
  • Verify you're using the correct password
  • Check for typos or case sensitivity
  • If you've forgotten the password, the database cannot be recovered
```

**Solution:** Use the correct password. If forgotten, restore from a backup.

---

### ODB-AUTH-002 — Password Too Short

```
Error ODB-AUTH-002: Password must be at least 8 characters long
```

**Solution:** Use a password with at least 8 characters.

---

### ODB-TABLE-001 — Table Not Found

```
Error ODB-TABLE-001: Table 'users' does not exist in database 'app.odb'

Suggestions:
  • Create the table first: db.createTable('users')
  • Enable auto-creation: OverDrive.open('app.odb', auto_create_tables=True)
  • Check for typos in the table name
```

**Solution:** Either create the table explicitly, or enable `auto_create_tables=True` (default).

---

### ODB-FFI-001 — Native Library Not Found

```
Error ODB-FFI-001: Could not load native library 'overdrive.dll'

Searched paths:
  • C:\project\overdrive.dll
  • C:\project\lib\overdrive.dll

Suggestions:
  • Reinstall the package: pip install overdrive-db
  • Download the binary from GitHub Releases
  • Verify your platform is supported (Windows x64, Linux x64/ARM64, macOS x64/ARM64)
```

**Solution:** Download the native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) and place it in your project directory.

---

### ODB-TXN-001 — Transaction Conflict

```
Error ODB-TXN-001: Transaction conflict detected — write-write conflict on table 'orders'

Suggestions:
  • Retry the transaction
  • Use transactionWithRetry() for automatic retry with backoff
  • Consider using a lower isolation level
```

**Solution:** Use `transactionWithRetry()` to automatically retry on conflicts.

---

## Retry Pattern

```python
from overdrive import OverDrive, TransactionError
import time

def with_retry(fn, max_retries=3):
    """Execute fn with exponential backoff retry on TransactionError."""
    for attempt in range(max_retries):
        try:
            return fn()
        except TransactionError:
            if attempt == max_retries - 1:
                raise
            delay = 0.1 * (2 ** attempt)  # 0.1s, 0.2s, 0.4s
            time.sleep(delay)

# Or use the built-in method:
result = db.transaction_with_retry(
    lambda txn: db.insert("orders", {"item": "widget"}),
    max_retries=3
)
```
