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
trap "rm -f tests/unit.rs" EXIT
ln -s unit/account_create.rs tests/unit.rs
cargo test --test unit --no-fail-fast
'
```

### Integration Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c '
trap "rm -f tests/integration.rs" EXIT
ln -s integration/create_account.rs tests/integration.rs
cargo test --test integration --no-fail-fast -- --include-ignored
'
```

### E2E Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c '
trap "rm -f tests/e2e.rs" EXIT
ln -s e2e/account_create.rs tests/e2e.rs
cargo test --test e2e --no-fail-fast -- --include-ignored
'
```

### Full Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c '
set -e
trap "rm -f tests/unit.rs tests/integration.rs tests/e2e.rs" EXIT

ln -s unit/account_create.rs tests/unit.rs
ln -s integration/create_account.rs tests/integration.rs
ln -s e2e/account_create.rs tests/e2e.rs

cargo test --tests --no-fail-fast -- --include-ignored
'
```