---
id: TASK-22
title: Make batch extraction failures script-safe
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
labels:
  - cli
  - reliability
dependencies: []
modified_files:
  - src/main.rs
  - tests/cli_output.rs
  - README.md
priority: high
ordinal: 22000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Clarify and enforce exit behavior for multi-file extraction when some or all inputs fail, while preserving the stdout data-only contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 If all extraction inputs fail, the command exits non-zero.
- [ ] #2 If some inputs fail and some succeed, exit behavior is explicit and documented.
- [ ] #3 stderr reports succeeded and failed counts for batch runs with failures.
- [ ] #4 stdout remains data-only and contains only successful extraction results.
- [ ] #5 Tests cover all-success, partial-failure, and all-failure batch scenarios.
<!-- AC:END -->
