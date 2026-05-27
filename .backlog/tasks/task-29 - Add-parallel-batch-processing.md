---
id: TASK-29
title: Add parallel batch processing
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 00:44'
labels:
  - feature
  - batch
  - performance
dependencies: []
modified_files:
  - src/main.rs
  - src/input.rs
  - src/output.rs
  - src/config.rs
  - src/agent_context.rs
  - README.md
priority: medium
ordinal: 31000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use the existing processing.parallelism config direction to process multiple input files concurrently while preserving deterministic output semantics where needed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Batch extraction can process more than one file concurrently when configured.
- [ ] #2 processing.parallelism controls concurrency or is replaced with an accurately named active setting.
- [ ] #3 stdout output remains parseable for JSON, JSON array, YAML, and CSV modes.
- [ ] #4 Failure handling remains script-safe for partial and total failures.
- [ ] #5 Tests cover ordering or explicitly documented ordering behavior.
<!-- AC:END -->
