---
id: TASK-70
title: Stream NDJSON batch output as records complete
status: Done
assignee: []
created_date: '2026-05-27 00:12'
updated_date: '2026-05-27 04:00'
labels:
  - code-review
  - performance
  - cli
dependencies: []
references:
  - src/main.rs
priority: low
ordinal: 72000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs accumulates every ExtractionResult into a Vec before calling render_results, even on the NDJSON path where each record is independent. Lifting NDJSON emission inside the per-file loop frees memory and lets downstream pipeline consumers start processing earlier. --json-array still requires accumulation. Reviewer: performance.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 NDJSON path prints each result as it completes; no Vec accumulation
- [x] #2 --json-array (and any explicit array shape) preserves today's accumulate-then-print behavior
- [x] #3 Tests assert ordering and per-line valid JSON
<!-- AC:END -->



## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added output::is_ndjson_stream() predicate (true only for batch + JSON + !json_array) and output::write_ndjson_record() which writes one compact JSON line and flushes stdout immediately. main.rs branches on this: streaming path bypasses Vec accumulation and emits per-record as extractions complete; --json-array path unchanged. Determining factor is input count, not success count — gives stable shape regardless of partial failures. Unit tests cover the predicate matrix and prove streamed output byte-matches batch render.
<!-- SECTION:NOTES:END -->
