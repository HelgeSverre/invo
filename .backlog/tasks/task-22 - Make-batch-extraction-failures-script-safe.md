---
id: TASK-22
title: Make batch extraction failures script-safe
status: Done
assignee:
  - '@claude'
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
- [x] #1 If all extraction inputs fail, the command exits non-zero (exit 2; tightened from TASK-40)
- [x] #2 If some inputs fail and some succeed, exit behavior is explicit and documented (exit 3; documented in README Exit codes table)
- [x] #3 stderr reports succeeded and failed counts for batch runs with failures ("Batch: N succeeded, M failed (of K files).")
- [x] #4 stdout remains data-only and contains only successful extraction results (summary goes to stderr; existing data-only contract preserved)
- [x] #5 Tests cover all-success, partial-failure, and all-failure batch scenarios (unit tests on decide_batch_exit_code cover the contract; TASK-40 integration test covers all-fail end-to-end)
<!-- AC:END -->
