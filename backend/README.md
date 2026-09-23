# Backend

# [Tech Stack](https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/techstack.md)

## Backend

| Technology | Purpose                                                                   |
| ---------- | ------------------------------------------------------------------------- |
| Rust       | Primary programming language for the backend application                  |
| Axum       | Web framework for HTTP APIs, routing, requests, responses, and middleware |
| Tokio      | Asynchronous runtime for asynchronous I/O and task execution              |
| SQLx       | PostgreSQL database access, SQL queries, transactions, and migrations     |

## Database

| Technology | Purpose                                          |
| ---------- | ------------------------------------------------ |
| PostgreSQL | Primary relational database for application data |

## Infrastructure

| Technology | Purpose                                                                |
| ---------- | ---------------------------------------------------------------------- |
| Docker     | Containerization and consistent development and deployment environment |

---

# Directory Structure

```text
backend/
├── Cargo.toml
├── Cargo.lock
├── .env.example
│
├── docker/
│   ├── Dockerfile
│   ├── Dockerfile.test
│   └── postgres/
│       └── ...
│
├── compose/
│   ├── docker-compose.yml
│   ├── docker-compose.dev.yml
│   ├── docker-compose.test.yml
│   └── docker-compose.prod.yml
│
├── migrations/
│   └── ...
│
├── src/
│   ├── main.rs
│   │
│   ├── app/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── state.rs
│   │   └── router.rs
│   │
│   ├── shared/
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   ├── response.rs
│   │   ├── auth.rs
│   │   └── validation.rs
│   │
│   └── domains/
│       └── <domains>/
│ 
└── tests/
    ├── unit/
    ├── integration/
    └── e2e/
```

---

# Test Strategy

```
                       /\
                      /  \
                     /    \
                    / E2E  \
                   /________\
                  /          \
                 / Integration\
                /              \
               /________________\
              /                  \
             /       Unit         \
            /                      \
           /________________________\
          /                          \
         /     Static Code Test       \
        /                              \
       /________________________________\
```

# Docker Command

## Test Profile Command

### Build Command

```bash
docker compose -f backend/compose/docker-compose.test.yml build test
```

### Clean Test Profile Command

```bash
docker compose -f backend/compose/docker-compose.test.yml down --volumes --remove-orphans
```

### Static Code Test Command

```bash
# Fix Formatting
docker compose -f backend/compose/docker-compose.test.yml run --rm test cargo fmt --all

# Check Only
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c 'cargo fmt --all -- --check && cargo check --tests && cargo clippy --all-targets --all-features -- -D warnings && cargo test --tests --no-fail-fast -- --include-ignored'
```

### Unit Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c '
status=0
trap "rm -f tests/unit.rs" EXIT

for test_file in tests/unit/*.rs; do
    echo "========================================"
    echo "Running unit test: $test_file"
    echo "========================================"

    rm -f tests/unit.rs
    cp "$test_file" tests/unit.rs

    if ! cargo test --test unit --no-fail-fast; then
        status=1
    fi
done

exit $status
'
```

### Integration Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c '
status=0
trap "rm -f tests/integration.rs" EXIT

for test_file in tests/integration/*.rs; do
    echo "========================================"
    echo "Running integration test: $test_file"
    echo "========================================"

    rm -f tests/integration.rs
    cp "$test_file" tests/integration.rs

    if ! cargo test --test integration --no-fail-fast -- --include-ignored; then
        status=1
    fi
done

exit $status
'
```

### E2E Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c '
status=0
trap "rm -f tests/e2e.rs" EXIT

for test_file in tests/e2e/*.rs; do
    echo "========================================"
    echo "Running E2E test: $test_file"
    echo "========================================"

    rm -f tests/e2e.rs
    cp "$test_file" tests/e2e.rs

    if ! cargo test --test e2e --no-fail-fast -- --include-ignored; then
        status=1
    fi
done

exit $status
'
```

### Full Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c '
status=0
trap "rm -f tests/unit.rs tests/integration.rs tests/e2e.rs" EXIT

for test_file in tests/unit/*.rs; do
    echo "========================================"
    echo "Running unit test: $test_file"
    echo "========================================"

    rm -f tests/unit.rs
    ln -s "$test_file" tests/unit.rs

    if ! cargo test --test unit --no-fail-fast; then
        status=1
    fi
done

for test_file in tests/integration/*.rs; do
    echo "========================================"
    echo "Running integration test: $test_file"
    echo "========================================"

    rm -f tests/integration.rs
    ln -s "$test_file" tests/integration.rs

    if ! cargo test --test integration --no-fail-fast -- --include-ignored; then
        status=1
    fi
done

for test_file in tests/e2e/*.rs; do
    echo "========================================"
    echo "Running E2E test: $test_file"
    echo "========================================"

    rm -f tests/e2e.rs
    ln -s "$test_file" tests/e2e.rs

    if ! cargo test --test e2e --no-fail-fast -- --include-ignored; then
        status=1
    fi
done

exit $status
'
```