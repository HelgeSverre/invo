---
id: TASK-14
title: Add example-driven CLI help and agent context
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
  - README.md
priority: low
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Improve progressive discovery for humans and agents through examples and structured introspection.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Top-level and config help include copy-pasteable examples.
- [ ] #2 --json is added as the conventional machine-output flag or documentation explains why --format json remains.
- [ ] #3 invo agent-context --json is added or explicitly tasked out with command/config shape.
<!-- AC:END -->
