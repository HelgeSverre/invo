---
id: TASK-47
title: Document multi-file JSON output now emits NDJSON
status: To Do
assignee: []
created_date: '2026-05-27 00:08'
labels:
  - code-review
  - contract
  - docs
dependencies:
  - TASK-4
references:
  - src/main.rs
priority: high
ordinal: 49000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Old multi-file --json output emitted a sequence of pretty-printed JSON blobs (technically not valid JSON either). The new code emits one compact JSON object per line (NDJSON) for batches; --json-array preserves the array form. The shape change is silent and breaks consumers parsing the old format. Reviewer: api-contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CHANGELOG documents the format change and provides a one-line jq migration example
- [ ] #2 agent-context payload labels --json batch output as NDJSON (couples with the agent-context shape ambiguity fix)
- [ ] #3 README batch-output example reflects NDJSON
<!-- AC:END -->
