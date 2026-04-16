# OverDrive-DB InCode SDK — Examples

This directory contains working examples for all four language SDKs.

## Examples

| Example                    | Description                                      |
|----------------------------|--------------------------------------------------|
| `basic-crud/`              | Insert, get, update, delete, and query documents |
| `transactions/`            | Callback-based and manual transactions           |
| `password-protected/`      | Create and open encrypted databases              |
| `ram-engine/`              | In-memory database with snapshot/restore         |
| `watchdog/`                | File integrity monitoring                        |
| `full-text-search/`        | Full-text search across documents                |
| `batch-operations/`        | Bulk insert, update, and delete                  |

## Running Examples

### Python
```bash
cd examples/basic-crud
pip install overdrive-db
python example.py
```

### Node.js
```bash
cd examples/basic-crud
npm install overdrive-db
node example.js
```

### Java
```bash
cd examples/basic-crud
mvn compile exec:java
```

### Go
```bash
cd examples/basic-crud
go run example.go
```

## Prerequisites

All examples require the OverDrive native library to be installed. See the [Getting Started guide](../docs/quickstart.md) for installation instructions.
