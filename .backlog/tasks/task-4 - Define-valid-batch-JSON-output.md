---
id: TASK-4
title: Define valid batch JSON output
status: Done
assignee:
  - '@codex'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 13:29'
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
- [x] #1 Batch JSON is compact NDJSON via --json or a JSON array via an explicit flag.
- [x] #2 Pretty JSON does not break machine-readable batch mode.
- [x] #3 README no longer describes pretty multi-document JSON as NDJSON.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add explicit JSON mode flags for compact JSON/NDJSON and JSON array output while preserving --format compatibility.
2. Collect successful extraction results before rendering so batch JSON can be emitted as NDJSON or an array.
3. Move output rendering into testable helpers and add unit coverage for batch NDJSON, pretty single JSON, and JSON array mode.
4. Update README output/flag examples and run cargo test/build.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added --json, --json-array, and --pretty. The main loop now collects successful results before rendering so JSON batches are compact NDJSON by default/--json, or one valid array with --json-array. Pretty JSON is restricted to single-result output unless an explicit array is requested. Added render helper tests for compact single JSON, batch NDJSON despite --pretty, JSON arrays, and default single pretty JSON.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
TASK-4 defines the JSON batch contract: compact NDJSON for multiple JSON results, --json for compact machine JSON, --json-array for one array, and --pretty for single-result human output. README now documents the stream contract and examples. Verified with cargo test, cargo build, and CLI help smoke.
<!-- SECTION:FINAL_SUMMARY:END -->
