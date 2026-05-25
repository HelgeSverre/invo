---
id: TASK-25
title: Reduce config validation drift risk
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
labels:
  - config
  - maintainability
dependencies: []
modified_files:
  - src/config.rs
priority: medium
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Config validation currently combines serde deny_unknown_fields with manual allowlists for nicer error paths. Keep clear errors while reducing the chance that schema, structs, and allowlists drift apart.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unknown-key errors remain clear and path-specific.
- [ ] #2 Adding a config field requires fewer duplicated updates than today.
- [ ] #3 Config schema generation and validation agree on accepted keys.
- [ ] #4 Tests cover nested unknown keys and representative known keys.
<!-- AC:END -->
