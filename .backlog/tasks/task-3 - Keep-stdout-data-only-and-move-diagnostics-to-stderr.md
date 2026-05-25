---
id: TASK-3
title: Keep stdout data-only and move diagnostics to stderr
status: Done
assignee:
  - '@codex'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 13:26'
labels:
  - cli
  - agent-friendly
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
  - tests/cli_output.rs
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Ensure JSON/YAML/CSV output can be parsed from stdout without log or diagnostic lines.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tracing and log output go to stderr.
- [x] #2 Default extraction emits no info logs on stdout.
- [x] #3 A redirected-output test proves stdout contains only result data and stderr contains diagnostics.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect current tracing subscriber and output paths to identify stdout/stderr behavior.
2. Configure tracing/log diagnostics to write to stderr while leaving extraction results on stdout.
3. Add or run a CLI-level smoke that redirects stdout/stderr separately and proves diagnostics do not contaminate stdout.
4. Run cargo test/build and update task status.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Configured tracing_subscriber to write diagnostics to stderr explicitly. Added integration tests that capture stdout/stderr separately for verbose config validation and missing input diagnostics; stdout remains parseable/status-only while tracing and diagnostics appear on stderr.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
TASK-3 keeps diagnostics off stdout by sending tracing output to stderr and adding CLI capture tests for verbose validation and missing-input diagnostics. Verified with cargo test, cargo build, and a manual stdout/stderr redirection smoke.
<!-- SECTION:FINAL_SUMMARY:END -->
