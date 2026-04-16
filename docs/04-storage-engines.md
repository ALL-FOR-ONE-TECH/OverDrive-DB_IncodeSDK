# Storage Engines

OverDrive has 6 storage engines. Each is optimized for a different use case. You pick the right one when opening a database or creating a table.

---

## Engine Overview

| Engine | Type | Latency | Best For |
|--------|------|---------|----------|
| `Disk` | B+Tree on disk | ~1ms | General-purpose persistent storage |
| `RAM` | HashMap in memory | <1µs | Caching, sessions, hot data |
| `Vector` | HNSW graph | ~5ms | AI embeddings, similarity search |
| `Time-Series` | Chunked time-partitioned | ~2ms | Metrics, IoT, logs |
| `Graph` | Node/Edge store | ~3ms | Social networks, knowledge graphs |
| `Streaming` | Topic-based partitioned | ~1ms | Event queues, message brokers |

---

## Disk Engine (Default)

The default engine. Data is stored in a B+Tree on disk with 4KB pages.

**Features:**
- Persistent — survives restarts
- Full MVCC transactions
- Write-Ahead Log for crash recovery
- AES-256-GCM encryption support
- Full-text search
- Secondary indexes

```python
# Disk is the default — no need to specify
db = OverDrive.open("app.odb")
db = OverDrive.open("app.odb", engine="Disk")  # same thing

# Insert and query work normally
db.insert("users", {"name": "Alice", "age": 30})
results = db.query("SELECT * FROM users WHERE age > 25")
```

**When to use:** Almost everything. This is your go-to engine.

---

## RAM Engine

Stores data in a HashMap entirely in memory. Sub-microsecond reads.

**Features:**
- O(1) get/put operations
- Sub-microsecond read latency
- Full MVCC + WAL support
- Snapshot to disk and restore
- Dynamic memory limits based on system RAM
- Survives restarts via snapshot/restore

**Memory limits (auto-configured):**
| System RAM | Default Limit |
|-----------|---------------|
| ≥ 32 GB | 8 GB |
| ≥ 16 GB | 4 GB |
| ≥ 8 GB | 2 GB |
| < 8 GB | 1 GB |

### Full RAM Database

```python
from overdrive import OverDrive

# Open a full RAM database
cache = OverDrive.open("cache.odb", engine="RAM")

# Insert — stored in memory, not on disk
cache.insert("sessions", {"user_id": 123, "token": "abc123", "expires": "2026-12-31"})
cache.insert("sessions", {"user_id": 456, "token": "xyz789", "expires": "2026-12-31"})

# Query works the same
sessions = cache.query("SELECT * FROM sessions WHERE user_id = 123")
print(sessions)

# Check memory usage
usage = cache.memoryUsage()
print(f"Using {usage['mb']:.2f} MB of {usage['limit_bytes'] // 1024 // 1024} MB limit")
print(f"Utilization: {usage['percent']:.1f}%")

# Persist to disk (snapshot)
cache.snapshot("./backup/cache_snapshot.odb")
print("Snapshot saved!")

# Later — restore from snapshot
cache2 = OverDrive.open("cache_restored.odb", engine="RAM")
cache2.restore("./backup/cache_snapshot.odb")
print(f"Restored {cache2.count('sessions')} sessions")

cache.close()
cache2.close()
```

### Per-Table RAM Storage (Hybrid Mode)

Mix RAM tables (hot data) with Disk tables (persistent data) in one database:

```python
db = OverDrive.open("app.odb")  # Disk database

# Create a RAM table for hot cache data
db.createTable("rate_limits", engine="RAM")
db.createTable("sessions", engine="RAM")

# These tables are on disk (auto-created)
db.insert("users", {"name": "Alice"})
db.insert("orders", {"item": "Laptop", "qty": 1})

# RAM table — fast reads
db.insert("sessions", {"user_id": 1, "token": "abc", "ttl": 3600})
session = db.findOne("sessions", "user_id = 1")  # sub-microsecond

db.close()
```

### Node.js RAM Engine

```javascript
const { OverDrive } = require('overdrive-db');

const cache = OverDrive.open('cache.odb', { engine: 'RAM' });

cache.insert('sessions', { userId: 123, token: 'abc123' });

const usage = cache.memoryUsage();
console.log(`RAM: ${usage.mb.toFixed(2)} MB`);

cache.snapshot('./backup/cache.odb');
cache.close();
```

### Java RAM Engine

```java
try (OverDrive cache = OverDrive.open("cache.odb", new OpenOptions().engine("RAM"))) {
    cache.insert("sessions", Map.of("userId", 123, "token", "abc123"));

    OverDrive.MemoryUsage usage = cache.memoryUsage();
    System.out.printf("RAM: %.2f MB%n", usage.getMb());

    cache.snapshot("./backup/cache.odb");
}
```

### Go RAM Engine

```go
cache, _ := overdrive.Open("cache.odb", overdrive.WithEngine("RAM"))
defer cache.Close()

cache.Insert("sessions", map[string]any{"userId": 123, "token": "abc123"})

usage, _ := cache.MemoryUsageStats()
fmt.Printf("RAM: %.2f MB\n", usage.Mb)

cache.Snapshot("./backup/cache.odb")
```

**When to use:** Session storage, rate limiting, leaderboards, real-time counters, hot caches.

---

## Vector Engine

Stores vector embeddings and supports approximate nearest neighbor (ANN) search using HNSW (Hierarchical Navigable Small World) graphs.

**Features:**
- Cosine, Euclidean, and dot-product distance metrics
- Approximate nearest neighbor search
- Metadata stored alongside vectors

```python
db = OverDrive.open("vectors.odb", engine="Vector")

# Create a vector index (dimensions must match your embedding model)
# e.g., OpenAI text-embedding-3-small = 1536 dimensions
db.query("CREATE VECTOR INDEX ON embeddings DIMENSIONS 1536 METRIC cosine")

# Insert with embedding
embedding = [0.1, 0.2, 0.3, ...]  # your 1536-dim vector
db.insert("embeddings", {
    "text": "OverDrive is an embedded database",
    "embedding": embedding,
    "source": "docs"
})

# Search for similar vectors
query_vector = get_embedding("fast embedded database")
results = db.query(f"VECTOR SEARCH embeddings LIMIT 5 METRIC cosine")
```

**When to use:** Semantic search, recommendation systems, image similarity, RAG (Retrieval-Augmented Generation).

---

## Time-Series Engine

Optimized for time-stamped data with automatic chunking and aggregation.

**Features:**
- Time-partitioned storage
- Range queries by timestamp
- Aggregations: AVG, MIN, MAX, SUM, COUNT
- TTL-based data retention
- Efficient for high-frequency writes

```python
db = OverDrive.open("metrics.odb", engine="Time-Series")

# Create a time-series with 30-day TTL
db.query("CREATE TIMESERIES cpu_usage TTL 2592000")

# Insert measurements (timestamp auto-added if not provided)
db.insert("cpu_usage", {"value": 45.2, "host": "server-1"})
db.insert("cpu_usage", {"value": 67.8, "host": "server-1"})
db.insert("cpu_usage", {"value": 23.1, "host": "server-2"})

# Query a time range
results = db.query(
    "SELECT * FROM cpu_usage WHERE timestamp > 1700000000 AND timestamp < 1700086400"
)

# Aggregate — average per 5-minute window
agg = db.query(
    "SELECT AVG(value) FROM cpu_usage GROUP BY window(300)"
)
```

**When to use:** Application metrics, IoT sensor data, financial tick data, server monitoring.

---

## Graph Engine

Stores nodes and edges with typed relationships. Supports graph traversal and shortest path queries.

**Features:**
- Typed nodes and edges
- Property storage on nodes and edges
- Graph traversal queries
- Shortest path finding

```python
db = OverDrive.open("social.odb", engine="Graph")

# Create node types
db.query("CREATE NODE TYPE Person")
db.query("CREATE NODE TYPE Company")
db.query("CREATE EDGE TYPE WORKS_AT")
db.query("CREATE EDGE TYPE KNOWS")

# Create nodes
alice_id = db.query("CREATE NODE Person {\"name\": \"Alice\", \"age\": 30}")[0]["id"]
bob_id   = db.query("CREATE NODE Person {\"name\": \"Bob\",   \"age\": 25}")[0]["id"]
acme_id  = db.query("CREATE NODE Company {\"name\": \"ACME Corp\"}")[0]["id"]

# Create edges (relationships)
db.query(f"CREATE EDGE KNOWS FROM {alice_id} TO {bob_id} {{\"since\": 2020}}")
db.query(f"CREATE EDGE WORKS_AT FROM {alice_id} TO {acme_id} {{\"role\": \"Engineer\"}}")

# Traverse — find all people Alice knows
friends = db.query(f"MATCH (p:Person)-[KNOWS]->(friend) WHERE p.id = '{alice_id}'")

# Shortest path
path = db.query(f"SHORTEST PATH FROM {alice_id} TO {bob_id} VIA KNOWS")
```

**When to use:** Social networks, recommendation engines, fraud detection, knowledge graphs, dependency analysis.

---

## Streaming Engine

Topic-based message storage with consumer groups and offset tracking. Like a lightweight Kafka.

**Features:**
- Topics with partitions
- Consumer groups with offset commits
- Message retention policies
- At-least-once delivery semantics

```python
db = OverDrive.open("events.odb", engine="Streaming")

# Create a topic with 3 partitions, 7-day retention
db.query("CREATE TOPIC user_events PARTITIONS 3 RETENTION 604800")

# Publish messages
db.insert("user_events", {
    "event": "user_signup",
    "user_id": 123,
    "timestamp": 1700000000
})

db.insert("user_events", {
    "event": "purchase",
    "user_id": 123,
    "amount": 99.99,
    "timestamp": 1700000100
})

# Subscribe and consume
db.query("SUBSCRIBE user_events GROUP analytics FROM beginning")

# Poll for messages
messages = db.query("POLL user_events GROUP analytics MAX 10 TIMEOUT 1000")
for msg in messages:
    print(f"Event: {msg['event']} — User: {msg['user_id']}")

# Commit offset after processing
db.query("COMMIT OFFSET user_events GROUP analytics")
```

**When to use:** Event sourcing, audit logs, real-time analytics pipelines, microservice communication.

---

## Choosing the Right Engine

```
Is your data time-stamped metrics or sensor readings?
  → Time-Series

Do you need to find similar items (AI, recommendations)?
  → Vector

Do you need to traverse relationships (social graph, dependencies)?
  → Graph

Do you need a message queue or event stream?
  → Streaming

Do you need sub-microsecond reads for hot data?
  → RAM

Everything else?
  → Disk (default)
```

---

## Mixing Engines

You can use multiple engines in one application:

```python
# Main database on disk
app_db = OverDrive.open("app.odb")

# Cache in RAM
cache = OverDrive.open("cache.odb", engine="RAM")

# Metrics in time-series
metrics = OverDrive.open("metrics.odb", engine="Time-Series")

# Or mix per-table in one database
db = OverDrive.open("hybrid.odb")
db.createTable("users")           # Disk (default)
db.createTable("sessions", engine="RAM")      # RAM
db.createTable("events", engine="Streaming")  # Streaming
```
