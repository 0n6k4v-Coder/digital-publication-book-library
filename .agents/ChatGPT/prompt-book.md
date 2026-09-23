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
- Authentication design specification:
  https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/authentication.md
- Authorization design specification:
  https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/authorization.md
- Backend structure and tech stack:
  https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/backend/backend/README.md

Source of truth:
- account.md is the primary source of truth for Account domain requirements, security requirements, design decisions, data model, use cases, and API contract.
- authentication.md is the source of truth for Authentication domain requirements, security requirements, design decisions, data model, use cases, API contract, and integration contract.
- authorization.md is the source of truth for Authorization domain requirements, security requirements, RBAC, permissions, data model, use cases, and integration contract.
- When the objective crosses domain boundaries, all applicable source-of-truth design documents must be satisfied together.
- A dependency domain must be included only when the source-of-truth design explicitly requires it for the objective.
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

- Read the primary source-of-truth design document for the objective.
- Identify all source-of-truth design documents that the objective depends on.
- Read all required dependency-domain design documents.
- Treat all applicable source-of-truth design documents as authoritative for their respective domains.
- Identify only design blockers specific to this objective and its required domain dependencies.
- Do not treat unrelated domains, unrelated Use Cases, missing code, tests, infrastructure, deployment, or implementation details as blockers unless the applicable source of truth requires them for this objective.
- A design blocker exists when the objective cannot be implemented correctly without inventing or guessing a business rule, API behavior, security rule, authorization rule, integration contract, or data rule.
- Do not invent missing business, API, security, authorization, or integration behavior.
- Do not treat an implementation technique as a blocker when multiple compliant techniques are possible.
- If an industry standard requires a new business, API, security, authorization, or integration decision not defined by the applicable source of truth, report it as a design blocker.
- Verify that cross-domain contracts required by the objective are sufficiently defined before implementation.

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

- Research only technologies, standards, and security requirements relevant to the objective and its required domain dependencies.
- Include cross-domain standards when the objective depends on them.
- Prefer official documentation and primary standards sources.
- Verify current behavior for Rust, Axum, Tokio, SQLx, PostgreSQL, HTTP authentication/authorization, and applicable security standards as required by the objective.
- Do not research unrelated technologies, domains, or future Use Cases.
- Do not introduce new business or API behavior from research alone; report it as a design consideration or blocker when the source of truth does not define it.

- Output: Simple, clear, direct, explicit, and concise.
- Include only findings relevant to the objective.
```

```text
Task 4: Create the implementation plan.

- Implement only the requested objective.
- Use all applicable source-of-truth design documents and existing repository conventions.
- Do not invent business, API, security, authorization, integration, or data behavior.
- Define the minimum necessary production and test file change set.
- “Minimum necessary” means the fewest files required to implement the objective correctly and completely, including required cross-domain dependencies.
- Include files from another domain only when that domain is explicitly required by the source of truth for the objective.
- For every file, state:
  - new file
  - shared existing file requiring incremental extension
  - objective-specific existing file requiring modification
- State the domain and responsibility of every changed file.
- Identify existing functionality that must remain unchanged.
- For HTTP Use Cases, include every applicable repository test layer:
  - unit
  - integration
  - E2E
- Do not use one test layer as a substitute for another.
- Use existing Use Case tests as the project convention.
- Include required migrations, dependencies, configuration, or shared infrastructure only when they are necessary for the objective.
- Exclude unrelated files, refactors, migrations, dependencies, configuration, domains, and Use Cases.
- Define the implementation sequence across domains.
- Explicitly identify the dependency order between Authentication, Authorization, and the business domain when applicable.
- Map every relevant design ID from every applicable source-of-truth document to its implementation or test coverage.

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
- Do not change unrelated domains or Use Cases.
- Required cross-domain files are allowed when the objective depends on them.

2. Design Compliance
- Re-read every applicable source-of-truth design document.
- Verify requirements, security requirements, design decisions, data model, use case, API contract, HTTP behavior, RBAC rules, and integration contracts.
- Verify every relevant design ID across all applicable domains.
- Verify all cross-domain contracts required by the objective.
- Do not invent undefined business, API, security, authorization, or integration behavior.
- If required behavior is undefined, treat it as a design blocker.

3. Code Correctness
- Verify validation, filtering, pagination, ordering, queries, counts, transactions, errors, state, edge cases, and module integration relevant to the objective.
- Verify cross-domain request and response flow.
- Verify authentication and authorization boundaries where required.
- Verify business-domain rules remain enforced by the owning domain.
- Do not invent an ordering rule when the design does not define one.

4. Configuration
- Verify Cargo.toml, Cargo.lock, migrations, environment files, Docker/Compose files, paths, modules, and test targets.
- Change only what the objective and its required dependencies need.

5. Security
- Verify authentication, authorization, RBAC, validation, resource limits, secret handling, error exposure, logging, injection risks, token handling, cache behavior, and transport security as applicable to the objective.
- Verify 401 and 403 behavior where applicable.
- Never return or log passwords, password hashes, tokens, or other credentials.
- Verify authorization decisions are enforced server-side.
- Verify deny-by-default behavior where required.

6. Tests
- For every HTTP Use Case, verify all applicable layers:
  - Unit
  - Integration
  - E2E
- Do not treat one layer as a substitute for another.
- Verify every objective-specific success, failure, security, authentication, authorization, integration, and edge case defined by the applicable source of truth.
- Verify authentication behavior where required.
- Verify authorization behavior where required.
- Verify sensitive-field exclusion where required.
- Verify test discovery and configuration.

7. Validation
- Run available cargo fmt, cargo check, cargo clippy, unit, integration, E2E, and repository Docker/Compose checks.
- Run the checks required by every affected domain.
- Report unavailable checks explicitly.
- Do not fabricate results.

8. Correction
- Fix every objective-related issue found.
- Re-run affected checks after corrections.
- Re-check scope and design compliance.

9. Final Check
- Confirm every changed file is necessary.
- Confirm all applicable test layers exist.
- Confirm all required cross-domain dependencies are implemented.
- Confirm unrelated domains and Use Cases are unchanged.
- Confirm the implementation matches all applicable sources of truth and the implementation plan.

Do not commit, push, or modify Git history.
```

```text
Task 8. Generate the final output.

- For every file that was actually and necessarily changed or newly created for this objective, provide the exact repository file path.
- Include files from required dependency domains when they were necessary for the objective.
- Group the final files by domain when multiple domains were changed.
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
