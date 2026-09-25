# Admin Application

## Table of Contents

1. [Directory Structure](#directory-structure)
2. [Directory and File Purpose](#directory-and-file-purpose)
3. [Technology Stack](#technology-stack)

   1. [Frontend](#frontend)
   2. [Infrastructure](#infrastructure)

4. [Test Strategy](#test-strategy)
5. [Docker Command](#docker-command)

   1. [Docker Test Profile Command](#docker-test-profile-command)

---

# 1. Directory Structure

```text
/frontend/admin
├── /public
├── /src
│   ├── /components
│   ├── /layouts
│   ├── /pages
│   │   ├── /login
│   │   ├── /accounts
│   │   └── /roles
│   ├── /services
│   ├── /styles
│   ├── /types
│   ├── App.tsx
│   └── main.tsx
├── /tests
│   ├── /unit
│   ├── /integration
│   └── /e2e
├── /docker
│   ├── Dockerfile.test
│   ├── Dockerfile.dev
│   └── Dockerfile.prod
├── /compose
│   ├── docker-compose.test.yml
│   ├── docker-compose.dev.yml
│   └── docker-compose.prod.yml
├── .dockerignore
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

# 2. Directory and File Purpose

| Path                              | Purpose                                     |
| --------------------------------- | ------------------------------------------- |
| `public/`                         | Static assets                               |
| `src/components/`                 | Reusable UI components                      |
| `src/layouts/`                    | Shared application layouts                  |
| `src/pages/`                      | Application pages                           |
| `src/pages/login/`                | Admin authentication                        |
| `src/pages/accounts/`             | Administrator account management            |
| `src/pages/roles/`                | Role management                             |
| `src/services/`                   | Backend API communication                   |
| `src/styles/`                     | Application styles                          |
| `src/types/`                      | TypeScript types                            |
| `src/App.tsx`                     | Root application component                  |
| `src/main.tsx`                    | Application entry point                     |
| `tests/unit/`                     | Unit test suites                            |
| `tests/integration/`              | Integration test suites                     |
| `tests/e2e/`                      | End-to-end browser test suites              |
| `docker/Dockerfile.test`          | Test environment image configuration        |
| `docker/Dockerfile.dev`           | Development environment image configuration |
| `docker/Dockerfile.prod`          | Production environment image configuration  |
| `compose/docker-compose.test.yml` | Test environment services                   |
| `compose/docker-compose.dev.yml`  | Development environment services            |
| `compose/docker-compose.prod.yml` | Production environment services             |
| `.dockerignore`                   | Files excluded from Docker build context    |
| `package.json`                    | Project dependencies and scripts            |
| `tsconfig.json`                   | TypeScript configuration                    |
| `vite.config.ts`                  | Vite configuration                          |

---

# 3. Technology Stack

## Frontend

| Technology | Purpose                       |
| ---------- | ----------------------------- |
| React      | UI framework                  |
| Vite       | Development and build tooling |
| TypeScript | Type-safe development         |
| Native CSS | Styling                       |

## Infrastructure

| Technology | Purpose                                                                |
| ---------- | ---------------------------------------------------------------------- |
| Docker     | Containerization and consistent development and deployment environment |

---

# 4. Test Strategy

```text
                        /\
                       /  \
                      /    \
                     /Manual\
                    /________\
                   /          \
                  /    E2E     \
                 /______________\
                /                \
               /   Integration    \
              /____________________\
             /                      \
            /         Unit           \
           /__________________________\
          /                            \
         /       Static Checks          \
        /________________________________\
```

| Level | Name          | Purpose                                                                 |
| ----- | ------------- | ----------------------------------------------------------------------- |
| 1     | Static Checks | Type checking, linting, formatting, and build validation                |
| 2     | Unit          | Test individual functions, components, and utilities                    |
| 3     | Integration   | Test multiple components and application behavior together              |
| 4     | E2E           | Test complete user flows in a real browser                              |
| 5     | Manual        | Verify visual, usability, accessibility, and real-world user experience |

---

# 5. Docker Command

## 5.1 Docker Test Profile Command

Build test image

```bash
docker compose -f frontend/admin/compose/docker-compose.test.yml build test
```

Clean Up

```bash
docker compose -f frontend/admin/compose/docker-compose.test.yml down --volumes --remove-orphans
```

Static Code Auto Fix and Test

```bash
# Auto Fix
docker compose -f frontend/admin/compose/docker-compose.test.yml run --rm test sh -c \
'npm run format && npm run lint -- --fix'

# Test
docker compose -f frontend/admin/compose/docker-compose.test.yml run --rm test sh -c \
'npm run format:check && npm run typecheck && npm run lint && npm run build'
```

Full Unit Test Command

```bash
docker compose -f frontend/admin/compose/docker-compose.test.yml run --rm test \
npm run test:unit
```

Full Integration Test Command

```bash
docker compose -f frontend/admin/compose/docker-compose.test.yml run --rm test \
npm run test:integration
```

Full E2E Test Command

```bash
docker compose -f frontend/admin/compose/docker-compose.test.yml run --rm test \
npm run test:e2e
```

Full Frontend Test Profile

```bash
docker compose -f frontend/admin/compose/docker-compose.test.yml run --rm test sh -c \
'npm run format:check && npm run typecheck && npm run lint && npm run build && npm run test:unit && npm run test:integration && npm run test:e2e'
```
