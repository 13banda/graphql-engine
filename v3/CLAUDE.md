Use just to run the tests please

## Aisura / Remote Dev

For development on Aisura or shared remote machines, use `compose.isolated.yaml`
which supports multiple parallel sessions without port or container conflicts.

### Start a session

```bash
ENGINE_HOST_PORT=3000 JAEGER_HOST_PORT=4002 \
  docker compose -p engine-1 -f compose.isolated.yaml up -d --build
```

### Tear down a session

```bash
docker compose -p engine-1 -f compose.isolated.yaml down -v
```

### Port allocation

Each session needs unique host ports. Suggested ranges:

- `ENGINE_HOST_PORT`: 3000, 3001, 3002, …
- `JAEGER_HOST_PORT`: 4002, 4003, 4004, …

Check availability: `lsof -i :<port>` or `ss -tlnp | grep <port>`

### How it works

`compose.isolated.yaml` includes `docker-compose.yaml` with
`multi-session-override.yaml` applied on top. The override uses Docker Compose
`!reset`/`!override` tags to:

- Strip `container_name` so Docker auto-generates unique names per project
- Remove all fixed host port bindings to prevent conflicts
- Re-expose only `engine` and `jaeger` UI via env-var ports

---

# v3-engine — Agent Setup Guide

This section is for AI coding agents running on remote Linux servers. Follow
these steps literally to clone, set up, and run tests.

## Prerequisites

| Tool           | Version    | Notes                                                   |
| -------------- | ---------- | ------------------------------------------------------- |
| Nix            | Any recent | Already on the server — source the profile (see step 1) |
| Docker         | 24.0+      | Only needed for full integration tests (`just test`)    |
| Docker Compose | V2         | Included with Docker 24+                                |

Nix provides Rust, Cargo, and all other build dependencies automatically.

## Step-by-step Setup

### 1. Source the Nix environment

Do this before running any `cargo` or `just` commands:

```bash
source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
```

Add this to your shell init (`.bashrc` / `.zshrc`) or run it at the start of
every session.

### 2. Clone the repository

```bash
git clone git@github.com:hasura/v3-engine.git
cd v3-engine
```

### 3. Build and verify

```bash
cargo check
```

### 4. Run tests

For unit tests only (fast, no Docker required):

```bash
cargo test -p ddn-engine-local-dev
```

For full integration tests (requires Docker):

```bash
just test
```

## Test Commands

```bash
# Run all tests with Docker deps (starts containers automatically)
just test

# Run a specific test by name
just test -- -E 'test(my_test_name)'

# Run tests for one crate only (Docker deps must already be running for integration tests)
cargo test -p ddn-engine-local-dev

# Run tests in watch mode — reruns on code changes
just watch

# Start Docker services without running tests
just start-docker-test-deps

# Stop all Docker services and remove volumes
just stop-docker

# Update golden files after intentional response format changes
just update-golden-files

# Review insta snapshot diffs interactively
cargo insta review
```

## Test Suite Map

Understanding which tests validate which behavior is critical. **Running the
wrong suite will miss real breakage.**

### `just test` (full workspace with Docker)

The canonical test command. Runs `cargo nextest run` across all crates. Requires
Docker services.

Covers:

- GraphQL execution against PostgreSQL (via NDC connectors)
- GraphQL introspection
- Query explain / plan generation
- Plugin hooks (pre-parse, pre-NDC-request, pre-NDC-response)
- OpenDD metadata validation and resolution
- JSON:API protocol

### `cargo test -p ddn-engine-local-dev`

Golden snapshot tests for the SQL (`/v1/sql`) interface. Requires Docker
services for full coverage.

- Test files: `crates/cloud/ddn-engine-local-dev/tests/sql/`
- Snapshot files: `response.snap`, `explain.snap` (checked in, will fail if
  output changes)
- Calls `sql::execute::execute_sql()` **directly** — not via the HTTP route
  handler
- **Will catch**: changes to query results, column names, data types, error
  messages from `execute_sql()`
- **Will not catch**: HTTP status code changes, response envelope changes in
  `routes.rs`, Content-Type header changes

To update snapshots after an intentional change:

```bash
UPDATE_GOLDENFILES=1 cargo test -p ddn-engine-local-dev
# then review and accept diffs:
cargo insta review
```

### `cargo test -p sql`

**Runs internal tests only.** The `sql` crate (`crates/cloud/sql/`) tests SQL
parsing and execution internals, NOT HTTP response format.

### `cargo test -p multitenant-engine`

**Minimal coverage.** The `multitenant-engine` crate has few tests.

### Known gap: HTTP route handler has limited test coverage

The SQL route handler in `crates/cloud/ddn-engine-local-dev/src/routes.rs` and
`crates/cloud/v3-engine-multitenant/crates/multitenant-engine/src/routes/sql.rs`
has **limited test coverage**. Changes to:

- HTTP status codes for error cases
- The error response format returned to HTTP clients
- `Content-Type` headers
- The JSONL streaming response path in the route

...may not be caught by `just test`. If you touch `routes.rs`, manually verify
the HTTP behavior.

## Key Code Locations

### /v1/sql endpoint

| Component                 | File                                                                             |
| ------------------------- | -------------------------------------------------------------------------------- |
| Core SQL execution        | `crates/cloud/sql/src/execute.rs`                                                |
| Local dev route handler   | `crates/cloud/ddn-engine-local-dev/src/routes.rs`                                |
| Multitenant route handler | `crates/cloud/v3-engine-multitenant/crates/multitenant-engine/src/routes/sql.rs` |

## Docker Services (for `just test`)

`just test` starts these services (defined in `ci.docker-compose.yaml`):

| Service                             | Port      | Purpose                                                           |
| ----------------------------------- | --------- | ----------------------------------------------------------------- |
| `postgres`                          | 5432      | PostgreSQL 16 + PostGIS — test database, seeded with Chinook data |
| `jaeger`                            | 4317/4318 | OpenTelemetry trace collector                                     |
| `postgres_connector_ndc_v01`        | 8080      | NDC v0.1 postgres connector                                       |
| `postgres_connector`                | 8082      | NDC v0.2 postgres connector                                       |
| `postgres_promptql`                 | 8083      | JDBC postgres connector                                           |
| `custom_connector`                  | 8102      | Reference NDC connector (relationships enabled)                   |
| `custom_connector_no_relationships` | 8103      | Reference NDC connector (no relationships)                        |
| `custom_connector_ndc_v01`          | 8101      | Reference NDC connector (v0.1)                                    |
| `auth_hook`                         | 3060      | Authentication webhook                                            |
| `pre_ndc_request_plugin_example`    | 5001      | Pre-NDC-request plugin                                            |
| `pre_ndc_response_plugin_example`   | 5004      | Pre-NDC-response plugin                                           |
| `pre_parse_plugin_example`          | 5003      | Pre-parse plugin                                                  |
| `test_data_server`                  | 8090      | nginx serving test fixtures                                       |

## Troubleshooting

### `cargo` or `nix` not found

```bash
source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
```

### Docker build is very slow (10+ minutes)

Normal on first run — the Rust Dockerfiles compile from scratch. Docker layer
caching means rebuilds are fast. Don't cancel; wait it out.

### Port conflicts

If ports 5432, 8080, 8082, etc. are already in use:

```bash
just stop-docker   # stops and removes all test containers and volumes
```

### Postgres health check times out

The postgres container runs init SQL scripts on first start. Check its logs:

```bash
docker compose -f ci.docker-compose.yaml logs postgres
```

### Connector not ready / "connection refused" on ports 8080–8103

```bash
docker compose -f ci.docker-compose.yaml ps
docker compose -f ci.docker-compose.yaml logs postgres_connector
```

### Snapshot tests fail with unexpected diffs

If you made an intentional change to SQL output, update the snapshots:

```bash
UPDATE_GOLDENFILES=1 cargo test -p ddn-engine-local-dev
cargo insta review   # review and accept each diff
```

If the diff is unexpected, the change broke the SQL response format —
investigate before accepting.
