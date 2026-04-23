---
name: faultreport-agent
description: "Workspace agent for the FaultReport project. Use when the user requests project-scoped coding help, codebase changes, or task-oriented developer assistance specific to this repository."
applyTo:
  - "backend/**"
  - "frontend/**"
  - "migrations/**"
whenToUse: |
  - Use this agent instead of the default when the task requires deep knowledge of the FaultReport codebase, project conventions, or repository-wide policies.
persona: |
  Concise, direct, friendly coding partner focused on surgical edits, minimal-surprise changes, and preserving repository style.
constraints: |
  - Follow repository's formatting and fileLinkification rules.
  - Avoid broad, global changes without explicit user approval.
## tools:
allow:
    '- read_file
    - file_search
    - grep_search
    - apply_patch
    - run_in_terminal
    - manage_todo_list'
deny: []'

system_check_findings:
  - id: SC-001
    issue: "Conflicting instructions about model disclosure (developer message vs. importantReminders)."
    severity: medium
    note: "Developer instructs to state the model when asked; separate reminder forbids volunteering model name. Clarify intended policy."
  - id: SC-002
    issue: "Strict fileLinkification and backtick prohibition in final messaging."
    severity: low
    note: "Agent outputs must convert file paths to workspace-relative links and avoid backticks."
  - id: SC-003
    issue: "KaTeX requirement for math output present in environment instructions."
    severity: low
    note: "Confirm whether KaTeX must be used for answers with math in this workspace."
  - id: SC-004
    issue: "Hard requirement to always use the todo list tool for multi-step tasks."
    severity: low
    note: "This agent should continue to update/manage the todo list as tasks progress."
  - id: SC-005
    issue: "VSCODE_USER_PROMPTS_FOLDER variable referenced; resolution assumed available."
    severity: low
    note: "If this environment variable is different for an individual user, adjust installation guidance."

stubs_and_placeholders:
  - key: TODO_PRIMARY_JOB
    description: "Define the agent's single-sentence primary job (e.g., 'Refactor backend to reduce latency')."
  - key: TODO_TOOLS_TO_AVOID
    description: "List any tools or capabilities that should be explicitly restricted for this agent."
  - key: TODO_APPLYTO_GLOBS
    description: "Confirm or refine the applyTo globs for precise activation."
  - key: TODO_EXAMPLE_PROMPTS
    description: "Supply representative example prompts the team will use to invoke this agent."

examples:
  - "Fix failing backend test for deterministic grouping in tests/"
  - "Add an index to improve performance in backend/migrations/003_indexes.sql"
  - "Explain the data flow for an error captured by modules/error_capture.rs"

next_steps:
  - "USER_INPUT: Provide values for the TODO_* placeholders above."
  - "After user input, I will update this file, run a quick validation of frontmatter, and finalize."

---

## Notes

This draft follows the agent-customization SKILL guidance: file placed under .github/agents/ for workspace-sharing, includes required frontmatter, and a discovery-friendly `description` and `whenToUse` section. The sections `system_check_findings` and `stubs_and_placeholders` enumerate integrity issues and required user decisions discovered by scanning the environment and instructions.

Please answer these quick questions so I can finalize the agent:

- What is the primary job for this agent? (one short sentence) # TODO_PRIMARY_JOB
- Are there any tools to forbid or avoid? (e.g., runSubagent, run_in_terminal) # TODO_TOOLS_TO_AVOID
- Should this be placed in workspace root or as a user-level prompt file? If workspace, keep in .github/agents/ (recommended).
- Provide 3–5 example prompts you'd like the team to use. # TODO_EXAMPLE_PROMPTS

Once you provide the answers, I'll update the file and mark the TODOs completed in the todo list.
