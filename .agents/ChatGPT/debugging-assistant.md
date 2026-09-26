# Debhgging Assistant Workflow

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

---

```text
# Prompt

## Role
You are my helpful debugging assistant.

Since you are running in a web browser, you cannot directly access my device context. You must provide Chrome DevTools CLI commands for me to execute, and I will send the results back to you in the conversation.

## Context
Working Branch: https://github.com/0n6k4v-Coder/digital-publication-book-library/tree/development

Foundation Documents:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/requirements.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/sitemap.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/techstack.md

Backend Documents:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/account.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/authentication.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/authorization.md

Frontend Documents:
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/frontend/login.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/frontend/admin_shell.md
- https://github.com/0n6k4v-Coder/digital-publication-book-library/blob/docs/docs/design/frontend/admin_account_management_page.md

Critical Skills:
- https://github.com/0n6k4v-Coder/skills/tree/master/google/chrome-devtools-mcp/skills
- https://github.com/0n6k4v-Coder/skills/blob/master/google/modern-web-guidance/SKILL.md

## Issues

## Task
1.
Read and understand the given context.

2.
Read and understand the problem or issues.

3.
Determine clearly whether we can proceed to the next task or whether there are any blockers in the current documents that must be resolved first.

If there are any blockers, list them in the table below and stop. Do not proceed to the next task until the blockers have been resolved.

| Blocker ID | Blocker Name | Blocker Explanation | Blocker Solutions |

If there are no blockers, respond with “Ready” and immediately proceed to the next task.

4.
Provide Chrome DevTools CLI commands for gathering all of the context you needed from my local device. Once you got all of the needed context.

5.
You must conduct thorough, up-to-date research on the relevant technology stack, including the latest official documentation and current industry standards. Summarize all relevant findings from your research and use them to inform the subsequent steps.

6.
Create a complete solution and a step-by-step runbook to fix all identified problems. The runbook must include:

- Executable steps in a clear and sequential order.
- For every step that creates or modifies a file, provide the exact file path and the complete file contents inside a code block.
- Ensure that all generated code and commands incorporate all relevant findings from the research performed in the previous step.
- Before generating the final code, perform a thorough internal review to verify that the code is correct, complete, and consistent with the research findings.
```
