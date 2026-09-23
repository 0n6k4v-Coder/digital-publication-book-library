# Prompt Book

```text
Simple, Clear, Direct, Explicit and Concise
```

```text
Please ensure the document is clear, concise, well-organized, and implementation-ready—serving as a reliable source of truth without unnecessary over-explanation.
```

```text
Give me a commit message and extended description.
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
Task 2: Verify design readiness for the objective.

- Read the relevant design documents.
- Treat the specified source-of-truth design document as authoritative for requirements, security, data model, use case, API contract, and HTTP behavior.
- Identify only design blockers specific to this objective.
- Do not treat missing code, tests, infrastructure, deployment, unrelated domains, or unrelated Use Cases as blockers unless the source of truth requires them for this objective.
- A design blocker exists when the objective cannot be implemented correctly without inventing or guessing a business rule, API behavior, security rule, or data rule.
- Do not invent missing business or API behavior.
- Do not treat an implementation technique as a blocker when multiple compliant techniques are possible.
- If an industry standard requires a new business or API decision not defined by the source of truth, report it as a design blocker.

If no design blockers exist, output only:

Ready

If design blockers exist, output only:

| ID | Details | Solutions |
|---|---|---|
| <Design ID or blocker ID> | <Specific blocker> | <Exact action required> |

Output must be simple, clear, direct, explicit, and concise.
```

```text
Task 3. Conduct deep research using the latest official documentation for the relevant technology stack and the latest applicable industry standards.
- Output: Simple, clear, direct, explicit, and concise.
- Include only findings relevant to the objective.
```

```text
Task 4: Create the implementation plan.

- Implement only the requested objective.
- Use the source-of-truth design and existing repository conventions.
- Do not invent business, API, security, or data behavior.
- Define the minimum necessary production and test file change set.
- “Minimum necessary” means the fewest files required to implement the objective correctly and completely, not the fewest files possible.
- For every file, state:
  - new file
  - shared existing file requiring incremental extension
  - objective-specific existing file requiring modification
- State the responsibility of every changed file.
- Identify existing functionality that must remain unchanged.
- For HTTP Use Cases, include every applicable repository test layer:
  - unit
  - integration
  - E2E
- Do not use one test layer as a substitute for another.
- Use existing Use Case tests as the project convention.
- Exclude unrelated files, refactors, migrations, dependencies, configuration, and Use Cases.
- Define the implementation sequence.
- Map each relevant design ID to its implementation or test coverage.

Output must be simple, clear, direct, explicit, and concise.
```

```text
Task 5: Apply the research findings to the implementation approach.
- Output: Simple, clear, direct, explicit, and concise.
```

```text
Task 6. Implement the plan and generate the required code.
```

```text
7. Review, Validate, and Correct the Generated Code.

Before final output, inspect the actual repository and correct all objective-related issues.

1. Scope
- Keep only files required for the objective.
- Remove unnecessary code, files, dependencies, migrations, configuration, refactors, and formatting-only changes.
- Do not change unrelated Use Cases.

2. Design Compliance
- Re-read the source-of-truth design.
- Verify requirements, security, design decisions, data model, use case, API contract, and HTTP status rules.
- Verify every relevant design ID.
- Do not invent undefined business or API behavior.
- If required behavior is undefined, treat it as a design blocker.

3. Code Correctness
- Verify validation, filtering, pagination, ordering, queries, counts, transactions, errors, state, edge cases, and module integration.
- Verify pagination behavior matches the design.
- Do not invent an ordering rule when the design does not define one.

4. Configuration
- Verify Cargo.toml, Cargo.lock, migrations, environment files, Docker/Compose files, paths, modules, and test targets.
- Change only what the objective requires.

5. Security
- Verify authentication, authorization, validation, resource limits, secret handling, error exposure, logging, injection risks, and cache behavior.
- Never return or log passwords, password hashes, tokens, or other credentials.

6. Tests
- For every HTTP Use Case, verify all applicable layers:
  - Unit
  - Integration
  - E2E
- Do not treat one layer as a substitute for another.
- For AC_UC_02, verify:
  - defaults
  - page/page_size validation
  - maximum page_size
  - status filtering
  - deleted-account handling
  - total count
  - pagination boundaries
  - empty results
  - authentication/authorization
  - cache headers
  - sensitive-field exclusion
- Verify test discovery and configuration.

7. Validation
- Run available cargo fmt, cargo check, cargo clippy, unit, integration, E2E, and repository Docker/Compose checks.
- Report unavailable checks explicitly.
- Do not fabricate results.

8. Correction
- Fix every objective-related issue found.
- Re-run affected checks after corrections.
- Re-check scope and design compliance.

9. Final Check
- Confirm every changed file is necessary.
- Confirm all applicable test layers exist.
- Confirm unrelated Use Cases are unchanged.
- Confirm the implementation matches the source of truth and the plan.

Do not commit, push, or modify Git history.
```

```text
Task 8. Generate the final output.
- For every file that was actually and necessarily changed or newly created for this objective, provide the exact repository file path.
- Provide the complete final code for each such file in a separate code block.
- Do not provide regenerated versions of unrelated files.
- Do not provide files that were not necessary for the objective.
- Do not provide partial code, diffs, or omitted sections.
```


```text
### Debugging Workflow

**Task 1: Analyze the Problem**

* Read the given context, error, logs, and relevant repository state.
* Identify the exact problem, root cause, and affected files.

**Task 2: Research**

* Research the latest official documentation for the relevant tech stack.
* Verify relevant current industry standards and recommended practices.
* Use authoritative sources and apply only findings relevant to the problem.

**Task 3: Generate the Fix**

* Apply the research findings to generate the required code or configuration changes.
* Keep changes limited to the problem being fixed.

**Task 4: Review and Validate**

* Review the generated fix for correctness, security, design compliance, and consistency with the repository.
* Verify paths, dependencies, configuration, runtime behavior, and tests.
* Fix any findings and repeat the relevant checks.

**Task 5: Final Output**

* Provide the exact file path for every changed file.
* Provide the complete final code block for every changed file.
* Do not provide partial code, diffs, or unnecessary changes.
* Do not commit or push unless explicitly requested.
```
