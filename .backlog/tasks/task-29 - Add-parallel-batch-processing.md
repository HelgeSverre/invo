---
id: TASK-29
title: Add parallel batch processing
status: Done
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 15:31'
labels:
  - feature
  - batch
  - performance
dependencies: []
modified_files:
  - src/main.rs
  - src/input.rs
  - src/output.rs
  - src/config.rs
  - src/agent_context.rs
  - README.md
priority: medium
ordinal: 31000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use the existing processing.parallelism config direction to process multiple input files concurrently while preserving deterministic output semantics where needed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Batch extraction can process more than one file concurrently when configured.
- [x] #2 processing.parallelism controls concurrency or is replaced with an accurately named active setting.
- [x] #3 stdout output remains parseable for JSON, JSON array, YAML, and CSV modes.
- [x] #4 Failure handling remains script-safe for partial and total failures.
- [x] #5 Tests cover ordering or explicitly documented ordering behavior.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented FIFO-streaming parallel batch processing in commit 8ecd4ff. New processing.parallelism u32 config field (default 1 preserves sequential semantics for existing users). When parallelism > 1, the batch loop uses futures::StreamExt::buffer_unordered to run extractions concurrently, sharing the single LlmContext (and thus the reqwest connection pool) across workers. Output emerges in completion order, not input order; per-record file.path + file.hash self-identify each result. Single-file mode unchanged — it bypasses the stream so anyhow errors propagate directly (AC #4). Tests added: default = 1, yaml parse, zero clamps at runtime. README has a new sub-section documenting the trade-off; CHANGELOG entry under Added. AC #5 ordering: documented as 'completion order' explicitly in README + CHANGELOG; no test for ordering itself because that's exactly the non-deterministic property. Test count: 124+16 -> 127+16. AC #2 — kept the parallelism name from the task spec; the field is real and active now.
<!-- SECTION:FINAL_SUMMARY:END -->
