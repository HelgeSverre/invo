---
id: TASK-6
title: Make config validation honest for missing files
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - config
  - cli
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/config/mod.rs
  - src/main.rs
priority: medium
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
config validate must distinguish an existing valid config file from a missing config path using built-in defaults.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Missing config path reports missing status and effective defaults.
- [ ] #2 Existing config path reports the path validated.
- [ ] #3 Machine-readable validation output is available or intentionally deferred consistently with config command design.
<!-- AC:END -->
