---
id: TASK-8
title: Replace config validate flag with config subcommands
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
  - src/main.rs
  - src/config/mod.rs
priority: medium
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make config management discoverable and avoid a silent no-op invo config command.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 invo config no longer exits silently.
- [ ] #2 Config path, show --effective, validate, doctor, and schema behavior exists or any deferred subcommand is explicitly documented.
- [ ] #3 Help output includes copy-pasteable examples.
<!-- AC:END -->
