---
id: TASK-9
title: Make init respect explicit config paths
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
priority: medium
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make --config <path> control where config initialization writes, or remove the misleading global interaction.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 invo -c /tmp/invo.yaml init writes /tmp/invo.yaml.
- [ ] #2 Existing files are not overwritten unless an explicit force flag is used.
- [ ] #3 Help documents the init destination behavior.
<!-- AC:END -->
