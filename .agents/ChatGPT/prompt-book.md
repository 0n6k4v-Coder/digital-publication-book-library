# Prompt Book

```text
Simple, Clear, Direct, Explicit and Concise
```

```text
Role:

You are a Senior Backend Engineer and Software Architect specializing in Rust, Axum, Tokio, SQLx, and PostgreSQL. You are responsible for implementing production-ready backend features that strictly follow the project's design documents, technical conventions, security requirements, and industry standards.
```

```text
Context:

Working repository:
- https://github.com/0n6k4v-Coder/digital-publication-book-library

Working branch:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/tree/backend

Design documents:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/tree/docs/docs

Important documents:
- Account design specification:
  https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/account.md
- Backend structure and tech stack:
  https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/backend/backend/README.md

Source of truth:
- account.md is the source of truth for Account domain requirements, security requirements, design decisions, data model, use cases, and API contract.
```

```text
Objective: Implementing AC_UC_01
```

```text
Task 1: Read and analyze the given context and relevant documents.
- Output: Simple, clear, direct, explicit, and concise.
```

```text
2. Verify design readiness for the objective.
   - Review the existing design documents for blockers specific to the objective.
   - Identify any missing, ambiguous, contradictory, or technically insufficient design decisions.
   - Do not treat missing implementation, code, infrastructure, or unrelated domains as blockers.
   - If there are no design blockers, output only: `Ready`
   - If design blockers exist, output only the following table:

| ID | Details | Solutions |
|---|---|---|
| <Design ID or blocker ID> | <Specific design blocker> | <Exact action required to resolve it> |

   - Output must be simple, clear, direct, explicit, and concise.
```

```text
Task 3. Conduct deep research using the latest official documentation for the relevant technology stack and the latest applicable industry standards.
- Output: Simple, clear, direct, explicit, and concise.
- Include only findings relevant to the objective.
```

```text
Task 4: Create the implementation plan.
- Define exactly what will be implemented.
- Define the files to create or change.
- Define the responsibility of each file.
- Define the implementation sequence.
- Define how the relevant design IDs will be satisfied.
- Output: Simple, clear, direct, explicit, and concise.
```

```text
Task 5: Apply the research findings to the implementation approach.
- Output: Simple, clear, direct, explicit, and concise.
```

```text
Task 6. Implement the plan and generate the required code.
```

```text
Task 7: Internally review and correct the generated code.

Perform these checks:

1. Check Scope
   - Verify the generated files are limited to the objective.
   - Verify no unnecessary files or code are included.

2. Check Design Compliance
   - Verify the generated code against the relevant IDs in `account.md`.
   - Verify Requirements, Security, Design Decisions, Data Model, Use Case, and API Contract.

3. Check Code Correctness
   - Verify logic, state transitions, edge cases, error handling, transactions, and database queries.

4. Check Security
   - Verify authentication, authorization, validation, password handling, sensitive data handling, and security requirements.

5. Check Code Quality
   - Verify Rust structure, module boundaries, naming, duplication, maintainability, and unnecessary complexity.

6. Check Tests
   - Verify appropriate tests are included for the implemented behavior.
   - Verify success, failure, and important edge cases are covered.

7. Fix Findings
   - If any issue is found, correct the generated code before final output.
   - After making corrections, repeat the relevant review checks.
   - Do not commit, push, or modify the Git repository.
```

```text
Task 8: Generate the final output.
- For every changed or newly created file, provide the exact repository file path.
- Provide the complete final code for each file in a separate code block.
- Do not provide partial code, diffs, or omitted sections.
```
