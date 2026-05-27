---
id: TASK-74
title: Isolate verbose tracing test from ambient RUST_LOG
status: Done
assignee: []
created_date: '2026-05-27 00:12'
updated_date: '2026-05-27 04:00'
labels:
  - code-review
  - test
dependencies: []
references:
  - tests/cli_output.rs
priority: low
ordinal: 76000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
tests/cli_output.rs verbose_config_validation_keeps_tracing_on_stderr expects -v alone to enable tracing. But src/main.rs:147 uses EnvFilter::try_from_default_env(), which makes -v a fallback only when RUST_LOG is unset. CI or developer environments with RUST_LOG=warn defeat the test silently. Reviewer: correctness.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Test invocation calls .env_remove("RUST_LOG") (or sets it explicitly to the expected value)
- [x] #2 Test passes regardless of ambient RUST_LOG
<!-- AC:END -->



## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added .env_remove("RUST_LOG") to verbose_config_validation_keeps_tracing_on_stderr. Verified test passes with ambient RUST_LOG=warn set.
<!-- SECTION:NOTES:END -->
