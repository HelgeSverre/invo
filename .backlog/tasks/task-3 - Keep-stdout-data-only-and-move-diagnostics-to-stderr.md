---
id: TASK-3
title: Keep stdout data-only and move diagnostics to stderr
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - cli
  - agent-friendly
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
  - src/pipeline.rs
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Ensure JSON/YAML/CSV output can be parsed from stdout without log or diagnostic lines.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tracing and log output go to stderr.
- [ ] #2 Default extraction emits no info logs on stdout.
- [ ] #3 A redirected-output test proves stdout contains only result data and stderr contains diagnostics.
<!-- AC:END -->
