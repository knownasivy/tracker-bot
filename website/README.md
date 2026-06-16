# Deploy Structure? (WIP)

Caddy
  └─Axum
    └─PgBouncer
      └─Postgres

# Project
main.rs
app.rs
db/
├── mod.rs
├── postgres.rs
└── migrations/

# TODO
* Etags for caching files etc
* Users like this?
```rust
#[derive(Clone)]
struct CurrentUser {
    name: String,
}
task_local! {
    pub static USER: CurrentUser;
}
```

# Create file with 

* File hashing for deduplication

# Keep allocations LOW (this matters more than anything)

Most Axum "slowdowns" come from String, Vec, cloning, JSON, etc.

Do:
Prefer &str over String in hot paths
Avoid .clone() in request flow
Use Arc<T> for shared state (not cloning big structs)

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Pool>,
}
```

Avoid:
cloning request state per handler
building large intermediate structs unnecessarily

# Use the right extractor strategy

Axum extractors are convenient, but they can allocate.

Faster pattern:

Prefer minimal extraction:

```rust
async fn handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
```

Instead of pulling huge structs like:

Json<VeryLargeStruct>

Only extract what you actually need.

# Avoid per-request locking

This is a big hidden killer.

Bad:
Mutex<HashMap<...>> for shared hot data
locking inside handlers
synchronous blocking inside async context
Better:
DashMap for concurrent maps
RwLock only when necessary
precomputed caches

# Use tower-http layers carefully

Middleware is powerful but can stack overhead.

Keep minimal:
tracing
compression (only if needed)
timeout
CORS (cheap)

# Spawn blocking work correctly

Never block the Tokio runtime.

Correct:
```rust
tokio::task::spawn_blocking(|| {
    heavy_cpu_work()
}).await?;
```

If you don’t:

you stall worker threads
latency spikes under load

# Database access is usually the REAL bottleneck

Rules:
always use connection pooling
avoid N+1 queries
batch inserts/reads
use prepared statements

# Don’t overuse async

async only for I/O (DB, network, file)
CPU stays sync or spawn_blocking

# Use tower for backpressure instead of custom logic

Instead of writing manual rate limiting / queues:
use tower::limit, buffer, timeout
These are optimized and composable.

# Setup postgresql

sudo apt update

sudo apt install postgresql postgresql-contrib

postgresql.conf
```conf
# CONNECTIONS
max_connections = 50 # 4 vcpu = 50, 4+ = 100

# MEMORY (Scale with mem)
shared_buffers = 2GB # 25% RAM
effective_cache_size = 6GB # 75% RAM
work_mem = 16MB
maintenance_work_mem = 512MB

# WAL
wal_buffers = 16MB
wal_compression = on

# CHECKPOINTS
checkpoint_completion_target = 0.9
min_wal_size = 1GB
max_wal_size = 4GB
checkpoint_timeout = 15min

# PERFORMANCE BEHAVIOR
random_page_cost = 1.1
effective_io_concurrency = 200

# OBSERVABILITY
shared_preload_libraries = 'pg_stat_statements'

# MISC
default_statistics_target = 100
```

# Setup pg bouncer
sudo apt install pgbouncer

/etc/pgbouncer/pgbouncer.ini
```ini
[databases]
appdb = host=127.0.0.1 port=5432 dbname=appdb

[pgbouncer]
listen_addr = 127.0.0.1
listen_port = 6432

pool_mode = transaction

max_client_conn = 5000

default_pool_size = 20
reserve_pool_size = 5

server_idle_timeout = 600
client_idle_timeout = 0

ignore_startup_parameters = extra_float_digits

auth_type = scram-sha-256
auth_file = /etc/pgbouncer/userlist.txt
```

# sqlx pool settings
```rust

PgPoolOptions::new()
    .max_connections(10)
    .min_connections(2)

```