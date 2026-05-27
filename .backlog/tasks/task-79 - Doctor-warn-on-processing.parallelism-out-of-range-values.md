---
id: TASK-79
title: Doctor-warn on processing.parallelism out-of-range values
status: Done
assignee: []
created_date: '2026-05-27 16:04'
updated_date: '2026-05-27 16:06'
labels:
  - config
  - doctor
  - task-29-followup
dependencies: []
references:
  - src/config.rs
priority: low
ordinal: 81000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
TASK-29 introduced processing.parallelism: u32 with default 1. The runtime clamps to .max(1) before feeding futures::buffer_unordered, so parallelism: 0 silently behaves like parallelism: 1 — which is fine but invisible to users.

Two warnings worth surfacing in config doctor:

1. parallelism: 0 — the user typed something they probably didn't mean. Doctor should warn and explain the clamp.

2. parallelism above some sanity threshold (e.g. > 32) — at high concurrency against rate-limited provider APIs the retries can stampede and waste budget. Doctor should warn and recommend lowering the value.

Surface: src/config.rs config_doctor_warnings function; pattern mirrors the existing ocr_doctor_warning and ocr_feature_gate_warning helpers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 config_doctor_warnings emits a warning when processing.parallelism == 0 explaining the runtime clamp to 1
- [x] #2 config_doctor_warnings emits a warning when processing.parallelism is greater than 32 recommending a lower value
- [x] #3 Unit tests cover both new warning paths and the silent path (1..=32 emits no parallelism warning)
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added parallelism_doctor_warning in src/config.rs. Warns when processing.parallelism == 0 (clamped to 1 at runtime — quiet for robustness but usually a typo) and when > 32 (PARALLELISM_SOFT_CAP — hosted providers' default per-key rate limits start dominating). Three new unit tests verify both warning paths and silence in the 1..=32 range. DoD #4 (docs) and #5 (notes) not applicable — this is a new warning surface that documents itself via the message; no user-facing API or output schema changed. Test count: 127+16 -> 130+16.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied and checked
- [x] #2 Relevant tests are added or updated
- [x] #3 cargo fmt, cargo test, and cargo clippy --all-targets -- -D warnings pass
- [ ] #4 Documentation/help/config examples are updated when behavior or user-facing output changes
- [ ] #5 Backlog task notes and final summary describe what changed and how it was verified
- [x] #6 Self-review confirms no unrelated changes, secrets, or debug artifacts are included
<!-- DOD:END -->
