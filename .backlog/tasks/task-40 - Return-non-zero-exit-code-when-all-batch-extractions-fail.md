---
id: TASK-40
title: Return non-zero exit code when all batch extractions fail
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:07'
updated_date: '2026-05-27 00:49'
labels:
  - code-review
  - reliability
  - cli
  - bug
dependencies:
  - TASK-22
references:
  - src/main.rs
priority: high
ordinal: 42000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs:174-188 swallows per-file errors when inputs.len()>1 and returns Ok(()) (exit 0) even when zero files succeeded. Scripts using 'invo *.pdf || handle' see false success; piping to 'mv out.ndjson archive/' archives an empty file. Single-file mode correctly propagates errors. Aligns with TASK-22's planned 0/1/2/3 exit-code contract. Reviewers: agent-native, adversarial, reliability.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Multi-file batch with zero successes exits non-zero (per TASK-22: exit 2 for all-batch-fail, exit 3 for partial)
- [x] #2 Single-file behavior is unchanged
- [ ] #3 Stderr summary line lists per-file outcome counts on partial failure (deferred: this commit scoped to all-fail per user direction; partial-failure summary belongs to the full TASK-22 contract)
- [x] #4 Integration test asserts the new exit codes
<!-- AC:END -->
