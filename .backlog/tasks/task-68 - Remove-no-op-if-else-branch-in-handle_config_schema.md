---
id: TASK-68
title: Remove no-op if/else branch in handle_config_schema
status: Done
assignee: []
created_date: '2026-05-27 00:11'
updated_date: '2026-05-27 00:44'
labels:
  - code-review
  - maintainability
  - refactor
dependencies: []
references:
  - src/config_commands.rs
priority: low
ordinal: 70000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/config_commands.rs::handle_config_schema still branches on --json even though schema output is JSON either way. The non-json branch writes a human label to stdout before JSON, which weakens the stdout=data-only contract for a schema-producing command.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 handle_config_schema always emits print_json(&schema)?; the if/else branch is removed
- [x] #2 If a human-readable label is desired, it is written to stderr
<!-- AC:END -->
