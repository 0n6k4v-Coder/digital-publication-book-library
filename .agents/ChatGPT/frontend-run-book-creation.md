```text
# Context

Working Repository:
- https://github.com/0n6k4v-Coder/digital-publication-book-library

Working Branch:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/tree/frontend

Target Application: `/frontend/admin`

Frontend Application Design:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/frontend/frontend/admin/README.md

Frontend Feature Design:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/frontend/login.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/frontend/admin-shell.md

Product Requirements:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/requirements.md

Backend Design:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/account.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/authentication.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/authorization.md

Backend Structure and API Context:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/backend/backend/README.md
```

```text
Source of Truth

Use each document according to its responsibility.

* The relevant frontend feature design document is the primary source of truth for frontend behavior.
* `frontend/admin/README.md` is the source of truth for Admin Application structure, technology stack, testing strategy, and frontend conventions.
* `requirements.md` is the source of truth for product scope and feature boundaries.
* `account.md` is the source of truth for Account business rules and Account API behavior.
* `authentication.md` is the source of truth for Authentication behavior, authentication API contracts, sessions, tokens, and authentication security.
* `authorization.md` is the source of truth for roles, permissions, and authorization behavior.
* Existing repository code is the source of truth for established frontend implementation conventions.
* Backend API contracts are authoritative for requests, responses, status codes, and error behavior.
* Frontend design documents must not redefine backend business or security rules.
* Do not duplicate backend business logic in the frontend.
* Do not invent undefined business, API, authentication, authorization, or security behavior.

When an objective crosses domain boundaries, all applicable source-of-truth documents must be satisfied together.

When applicable source-of-truth documents conflict, do not invent a resolution. Report the conflict as a design blocker.
```

```text
# Objective
```

`````text
# Task 1: Read and Analyze the Context

* Read the relevant frontend documentation.
* Read the relevant product requirements.
* Read all source-of-truth backend design documents required by the objective.
* Inspect the existing frontend repository.
* Inspect existing related pages, components, services, types, styles, and tests.
* Inspect the corresponding backend API implementation when necessary.
* Identify existing conventions that must be followed.
* Identify reusable frontend code.
* Identify existing API clients, request utilities, authentication handling, and error handling.
* Identify existing test conventions.

Output:

```text
Current Implementation
- <finding>

Required Behavior
- <finding>

Required Dependencies
- <finding>
```
Keep the output simple, clear, direct, explicit, and concise.
`````

````text
# Task 2: Verify Design Readiness

* Read the primary source-of-truth design document for the objective.
* Identify all required dependency-domain documents.
* Verify the frontend behavior can be implemented without inventing:

  * business rules
  * API behavior
  * authentication behavior
  * authorization behavior
  * security behavior
  * validation rules
  * state transitions
* Identify only blockers specific to the objective.

A design blocker exists when the frontend cannot be implemented correctly without guessing or inventing a required contract.

Do not treat these as blockers:

* implementation technique
* component organization preference
* styling technique
* testing implementation technique
* reusable component choice

If no design blockers exist, output only:

```text
Ready
```

If design blockers exist, output only:

| ID                        | Details            | Solutions               |
| ------------------------- | ------------------ | ----------------------- |
| <Design ID or blocker ID> | <Specific blocker> | <Exact action required> |
````

```
# Task 3: Conduct Deep Research

Before implementing any frontend code, you MUST use:

`modern-web-guidance`

Skill:
https://github.com/0n6k4v-Coder/skills/blob/master/google/modern-web-guidance/SKILL.md

The skill MUST be used before:

* implementing any HTML, CSS, or client-side JavaScript
* creating a new React component
* implementing frontend forms
* implementing frontend UI behavior
* implementing frontend layout or responsive behavior
* implementing browser-facing APIs
* implementing accessibility-related UI behavior

Follow the skill's required workflow:

1. Search for the relevant modern web guidance.
2. Retrieve the applicable guide(s).
3. Apply the relevant guidance to the implementation.
4. Adapt the guidance to the project's React, Vite, TypeScript, and Native CSS architecture.

Research only technologies, standards, and security requirements relevant to the objective.

Prefer official documentation and primary sources.

Relevant areas may include:

* React
* Vite
* TypeScript
* Browser APIs
* HTTP
* Authentication
* Authorization
* Accessibility
* Form validation
* Browser security
* Testing libraries
* Playwright
* Vitest
* Web standards

Verify current behavior when it affects implementation.

Do not introduce new product behavior based on research alone.

If research reveals a required product, security, API, or UX decision that is not defined by the source of truth, report it as a design consideration or blocker rather than inventing behavior.

Output only findings relevant to the objective.
```

````
# Task 4: Create the Implementation Plan

Implement only the requested objective.

Define the minimum necessary production file change set.

For every file, state:

* exact repository path
* new file or existing file
* domain / responsibility
* purpose
* required change

Classify files as:

```text
Page
Component
Layout
Service / API
Type
State / Hook
Style
Test
Configuration
```

Use existing abstractions before creating new ones.

Avoid unnecessary refactoring.

Do not introduce new libraries unless required by the objective or existing project conventions.

For every page or feature, explicitly define:

* route
* page structure
* components
* API calls
* request parameters
* response handling
* loading state
* empty state
* error state
* success state
* validation
* authorization behavior
* navigation behavior
* user actions
* confirmation behavior
* responsive behavior
* accessibility behavior
````

```
# Task 5: Define Backend Integration

For every API interaction:

* Identify the exact backend endpoint.
* Identify HTTP method.
* Identify request shape.
* Identify response shape.
* Identify applicable HTTP status codes.
* Identify error response behavior.
* Identify authentication requirements.
* Identify authorization requirements.
* Identify token/session behavior where applicable.
* Identify whether the operation mutates server state.
* Identify required UI invalidation or refresh behavior.

Do not implement business logic that belongs to the backend.

The frontend must treat the backend as the authority for:

* business rules
* account state
* authorization decisions
* lifecycle invariants
* credential validation
* security decisions
```

```
# Task 6: Define Authentication and Authorization Behavior

When the objective requires authentication:

* Determine how the authenticated state is obtained.
* Determine how access tokens are attached to protected API requests.
* Determine refresh behavior according to `authentication.md`.
* Determine logout behavior.
* Handle `401 Unauthorized` correctly.
* Do not expose tokens in the UI.
* Do not log credentials or tokens.
* Do not store credentials in places prohibited by the applicable security design.

When the objective requires authorization:

* Use server-side authorization as the authority.
* Respect permissions defined by `authorization.md`.
* Handle `403 Forbidden`.
* Do not infer permissions from UI state alone.
* Do not allow client-side role state to become the authorization authority.
* UI permission checks may control visibility, but backend authorization remains authoritative.
```

````
# Task 7: Implement UI Behavior

Implement the required user experience.

For every interactive state, handle explicitly:

```text
Initial
Loading
Success
Empty
Validation Error
API Error
Unauthorized
Forbidden
Mutation Pending
Mutation Success
Mutation Failure
```

For forms:

* Validate required fields.
* Match the backend contract.
* Display field-level errors when the API provides field-level failures.
* Display form-level errors when appropriate.
* Prevent duplicate submissions where required.
* Disable or protect destructive actions while pending.
* Preserve user input appropriately after errors.

For destructive actions:

* Require explicit confirmation when appropriate.
* Clearly identify the target and action.
* Handle failure without silently losing state.
````

```
# Task 8: Implement Responsive and Accessible UI

The implementation must follow the project's frontend conventions.

Verify:

* keyboard navigation
* visible focus
* semantic HTML
* labels for form controls
* accessible names
* error association
* button states
* loading states
* sufficient interaction feedback
* responsive layouts
* usable behavior on supported viewport sizes

Do not add a UI library unless required by the project.

Use Native CSS as defined by the Admin Application stack.
```

````
# Task 9: Implement Tests

For the objective, implement all applicable test layers:

## Static Checks

Verify:

* TypeScript
* lint
* formatting
* production build

## Unit

Test:

* pure functions
* utilities
* isolated components
* hooks
* validation logic

## Integration

Test:

* page behavior
* component interaction
* forms
* API interaction
* loading/error/success states
* authentication-aware behavior
* authorization-aware behavior

## E2E

Test complete user flows in a real browser.

For example:

```text
Login
  ↓
Navigate
  ↓
Perform Action
  ↓
Receive Result
  ↓
Verify UI
```

Do not use one test layer as a substitute for another.

Follow existing repository test conventions.
````

````
# Task 10: Review and Validate the Implementation

Before final output, inspect the actual repository and correct all objective-related issues.

## 1. Scope

* Keep only files required for the objective.
* Remove unnecessary changes.
* Do not refactor unrelated code.
* Do not modify unrelated pages or domains.

## 2. Design Compliance

* Re-read every applicable source-of-truth document.
* Verify all relevant design IDs.
* Verify API contracts.
* Verify authentication contracts.
* Verify authorization contracts.
* Verify security requirements.
* Verify integration contracts.

## 3. UI Correctness

Verify:

* routes
* navigation
* component behavior
* forms
* validation
* loading
* empty states
* errors
* success states
* destructive actions
* responsive behavior
* accessibility

## 4. API Correctness

Verify:

* endpoint
* HTTP method
* request
* response
* status handling
* error handling
* authentication
* authorization

## 5. Security

Verify:

* credentials are never exposed in UI
* tokens are never logged
* sensitive values are not rendered
* authentication failures are handled correctly
* authorization failures are handled correctly
* client-side permission checks do not replace server authorization

## 6. Tests

Verify all applicable:

* Unit
* Integration
* E2E

Verify:

* success cases
* failure cases
* validation cases
* authentication cases
* authorization cases
* edge cases
* loading states
* empty states
* sensitive-data handling

## 7. Validation

Run the available repository checks:

```text
TypeScript
Lint
Format
Build
Unit
Integration
E2E
Docker / Compose checks when affected
```

Do not fabricate results.

Report unavailable checks explicitly.

## 8. Correction

Fix all objective-related issues found.

Re-run affected checks after correction.
````

```
# Task 11: Verify Objective Completeness

Confirm:

* Every required page or component is implemented.
* Every required API interaction is implemented.
* Every required user action is implemented.
* Every required state is handled.
* Authentication behavior is implemented where required.
* Authorization behavior is implemented where required.
* All applicable tests exist.
* All applicable tests pass.
* No unrelated functionality changed.
* No undefined behavior was invented.
```

````
# Task 12: Generate the Complete Implementation Output

For every file created or modified for the objective, provide:

1. Exact repository path.
2. File status:
   - New
   - Modified
3. Complete final file content.
4. The implementation must include all applicable findings from Task 3.
5. The implementation must comply with all applicable source-of-truth documents.
6. The implementation must not contain placeholders, omissions, ellipses, pseudo-code, or comments such as:
   - `...`
   - `TODO`
   - `implement here`
   - `same as above`
   - `existing code`
   - `rest of code`

Output every file independently.

Required format:

## <Exact Repository Path>

**Status:** New | Modified

```<language>
<COMPLETE FINAL FILE CONTENT>
```

Repeat this format for every changed file.

Rules:

* Do not output a patch instead of the full file.
* Do not output only snippets.
* Do not omit unchanged sections of a modified file.
* Do not replace code with explanations.
* Do not hide implementation inside an archive.
* Do not require the user to open another artifact to see the source code.
* The code shown must represent the final implementation after applying the research findings, source-of-truth requirements, API contracts, security requirements, accessibility requirements, and testing requirements.
````

````
# Task 13: Generate the Required Test Commands

Use the repository's existing test-command style.

Provide commands for every applicable test layer:

### Static Checks

```bash
<command>
```

### Unit

```bash
<command>
```

### Integration

```bash
<command>
```

### E2E

```bash
<command>
```

### Docker / Compose

```bash
<command>
```

Only provide commands relevant to the objective.
````

```
# Final Rules

* Implement only the requested objective.
* Follow the frontend source of truth.
* Follow backend API contracts.
* Do not invent business rules.
* Do not duplicate backend business logic.
* Do not invent authentication or authorization behavior.
* Use the backend as the authority for business and security decisions.
* Reuse existing frontend abstractions.
* Avoid unnecessary dependencies.
* Avoid unrelated refactoring.
* Keep the implementation production-ready.
* Keep the output simple, clear, direct, explicit, and concise.
* Do not commit.
* Do not push.
* Do not modify Git history.
* Always provide the exact repository path for every created or modified file.
* Always provide the complete final source code for every created or modified file.
* The final response must contain the full implementation source, not only a summary, patch, diff, or archive.
* A patch or archive may be provided as an additional artifact, but never as a replacement for the full source code.
* Apply all relevant deep-research findings directly to the implementation before presenting the final source.
* MUST use the `modern-web-guidance` skill before implementing frontend code.
* MUST apply relevant findings from `modern-web-guidance` to the final implementation.
* MUST NOT bypass the skill when implementing HTML, CSS, client-side JavaScript, React UI, forms, accessibility, responsive behavior, or browser-facing functionality.
```
