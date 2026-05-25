---
id: TASK-7
title: Reject unknown config keys in strict validation
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - config
  - validation
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/config/schema.rs
  - src/config/mod.rs
priority: medium
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prevent typos such as identitty from silently passing config validation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Strict validation reports unknown top-level and nested keys.
- [ ] #2 Error messages include the invalid key path.
- [ ] #3 The active config template passes strict validation.
<!-- AC:END -->
