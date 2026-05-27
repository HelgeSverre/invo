---
id: TASK-60
title: Route 'invo init' success message to stderr to honor stdout=data-only contract
status: Done
assignee: []
created_date: '2026-05-27 00:10'
labels:
  - code-review
  - bug
  - cli
  - agent
dependencies:
  - TASK-3
references:
  - src/main.rs
priority: medium
ordinal: 62000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs:218 prints 'Created config file at <path>' via println! (stdout). agent-context (and TASK-3) document stdout as data-only. The init success message belongs on stderr (or, with --json, as a structured object on stdout). Reviewer: agent-native.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 init success message is emitted via eprintln! (stderr) by default
- [ ] #2 Optional: 'invo init --json' emits {"path": "...", "created": true} on stdout for programmatic callers
- [x] #3 Integration test asserts the message appears on stderr, not stdout
<!-- AC:END -->
