---
id: TASK-11
title: Make provider override a true per-run override
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - llm
  - cli
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/llm/mod.rs
  - src/main.rs
  - README.md
priority: medium
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provider override should work without requiring that provider to already be enabled, or the flag/docs should be renamed to match actual behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 --provider openai can select an OpenAI provider entry even if disabled, or docs/flag name are changed to match actual behavior.
- [ ] #2 Errors tell the user exactly what config or environment value is missing.
- [ ] #3 README examples match the implemented semantics.
<!-- AC:END -->
