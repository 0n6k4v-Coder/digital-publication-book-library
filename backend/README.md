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
docker compose -f backend/compose/docker-compose.test.yml run --rm test sh -c 'cargo fmt --all -- --check && cargo check --tests && cargo clippy --all-targets --all-features -- -D warnings'
```

### Unit Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test cargo test --test unit --no-fail-fast
```

### Integration Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test cargo test --test integration --no-fail-fast -- --include-ignored
```

### E2E Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test cargo test --test e2e --no-fail-fast -- --include-ignored
```

### Full Test Command

```bash
docker compose -f backend/compose/docker-compose.test.yml run --rm test cargo test --tests --no-fail-fast -- --include-ignored
```