---
id: TASK-46
title: Add CHANGELOG entry for removed 'invo config --validate' flag
status: To Do
assignee: []
created_date: '2026-05-27 00:08'
labels:
  - code-review
  - contract
  - docs
  - cli
dependencies:
  - TASK-8
references:
  - src/main.rs
priority: high
ordinal: 48000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
TASK-8 replaced the boolean flag 'invo config --validate' with nested subcommands ('invo config validate', 'invo config show', 'invo config path', 'invo config doctor', 'invo config schema'). Scripts calling the old form get a Clap parse error with no migration hint. Reviewer: api-contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CHANGELOG documents the old → new mapping
- [ ] #2 Optional: accept the legacy --validate flag as a deprecated alias that emits a stderr warning and dispatches to 'config validate' for one release
- [ ] #3 README/REVIEW-CLI usage examples reference the new subcommand form only
<!-- AC:END -->
