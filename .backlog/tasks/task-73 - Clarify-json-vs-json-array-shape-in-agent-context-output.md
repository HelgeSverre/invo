---
id: TASK-73
title: Clarify --json vs --json-array shape in agent-context output
status: Done
assignee: []
created_date: '2026-05-27 00:12'
updated_date: '2026-05-27 00:56'
labels:
  - code-review
  - docs
  - agent
dependencies:
  - TASK-14
references:
  - src/agent_context.rs
priority: low
ordinal: 75000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/agent_context.rs advertises --json and --json-array as batch options without explaining that --json on multiple files emits NDJSON (compact JSON-per-line) while --json-array emits a single JSON array. Agents calling --json on receipts/*.pdf and feeding the output to json.loads hit 'extra data' errors and diagnose the wrong root cause.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 agent-context labels --json output as 'NDJSON on batch, single object on single file'
- [ ] #2 agent-context labels --json-array as 'always a JSON array'
- [ ] #3 agent-context examples show both invocations with the resulting shape
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added json_flag_shape and json_array_flag_shape blocks to agent-context payload with single_file/batch shape descriptions; rewrote examples to annotate each invocation with its resulting shape.
<!-- SECTION:NOTES:END -->
