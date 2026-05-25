---
id: TASK-10
title: Make global flags usable with subcommands
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - cli
  - usability
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
priority: medium
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reduce CLI friction around config flag placement for subcommands.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 --config works before or after applicable subcommands, or help clearly documents the accepted placement.
- [ ] #2 Validation examples use the supported placement.
- [ ] #3 Clap tests or snapshots cover the chosen behavior.
<!-- AC:END -->
