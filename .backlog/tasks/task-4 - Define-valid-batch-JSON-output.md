---
id: TASK-4
title: Define valid batch JSON output
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - cli
  - output
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
  - README.md
priority: high
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make multi-file output parseable and align README examples with the implemented stream contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Batch JSON is compact NDJSON via --json or a JSON array via an explicit flag.
- [ ] #2 Pretty JSON does not break machine-readable batch mode.
- [ ] #3 README no longer describes pretty multi-document JSON as NDJSON.
<!-- AC:END -->
