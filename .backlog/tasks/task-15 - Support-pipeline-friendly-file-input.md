---
id: TASK-15
title: Support pipeline-friendly file input
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - cli
  - automation
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
  - src/pdf/mod.rs
priority: low
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add stdin and file-list input options for scripts and agents.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 invo - accepts PDF bytes from stdin or returns an actionable unsupported error.
- [ ] #2 --files-from <path> supports newline-delimited file lists.
- [ ] #3 Metadata for stdin input is deterministic.
<!-- AC:END -->
