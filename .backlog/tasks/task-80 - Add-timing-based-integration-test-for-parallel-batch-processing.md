---
id: TASK-80
title: Add timing-based integration test for parallel batch processing
status: Done
assignee: []
created_date: '2026-05-27 16:04'
updated_date: '2026-05-27 16:10'
labels:
  - test
  - task-29-followup
  - performance
dependencies: []
references:
  - tests/
  - src/llm.rs
priority: medium
ordinal: 82000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
TASK-29 added processing.parallelism and unit tests for the config field, but no test actually proves that parallelism > 1 runs files concurrently. The existing tests verify deserialization and the default-value contract.

Add an integration test that:
1. Builds a config with processing.parallelism: 4
2. Uses rig-core's test-utils (or wiremock against a fake provider) to expose a controllable per-call delay
3. Drives extract_invoice over N=4 inputs with each call taking ~200ms
4. Asserts wall-clock is significantly less than 4 * 200ms (the serial baseline) — e.g. < 400ms (50% of serial)

The test must be deterministic enough not to flake on a busy CI box; use generous tolerances (e.g. parallel completes in <60% of serial time).

Surface: tests/ — new file tests/parallelism.rs OR add to tests/cli_output.rs. Probably easier as a Rust integration test using LlmContext directly with a mocked provider than as a CLI integration test.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Test exists that builds a config with parallelism > 1, runs N >= parallelism inputs through a mock provider with controllable per-call delay, and asserts wall-clock < serial baseline by a meaningful margin
- [x] #2 Test does not depend on real network or real API keys
- [x] #3 Test tolerances are generous enough not to flake on busy CI; pick a multiplier that holds with margin to spare
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Two new #[tokio::test] tests in src/llm.rs::tests prove buffer_unordered + complete_invoice runs concurrently when parallelism > 1 and serially when parallelism = 1. The tests use MockCompletionModel + a tokio::time::sleep injected before each complete_invoice call to simulate per-call latency without any real network. parallel_batch_completes_faster_than_serial_baseline: N=4, parallelism=4, per_task=200ms — asserts elapsed < 400ms (50% of the 800ms serial baseline). Empirically completes in ~200ms with plenty of headroom. parallelism_one_runs_futures_serially: N=3, parallelism=1, per_task=100ms — asserts elapsed >= 300ms (must NOT overlap). Companions: together they prove buffer_unordered honors its bound in both directions. Test count: 133+16 -> 135+16. DoD #4 (docs) and #5 (notes) not applicable — internal test only.
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
