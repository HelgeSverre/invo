---
id: TASK-14
title: Add example-driven CLI help and agent context
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 14:43'
labels:
  - cli
  - agent-friendly
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
  - README.md
  - tests/cli_output.rs
priority: low
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Improve progressive discovery for humans and agents through examples and structured introspection.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Top-level and config help include copy-pasteable examples.
- [x] #2 --json is added as the conventional machine-output flag or documentation explains why --format json remains.
- [x] #3 invo agent-context --json is added or explicitly tasked out with command/config shape.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add copy-pasteable Clap help examples for extraction, config, init, and agent context. 2. Keep --json as the conventional machine-output flag across relevant commands. 3. Add invo agent-context --json with a structured CLI/config introspection payload and tests.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added example-driven help text and a structured agent-context command.

Made --json global for machine-readable command output where supported.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added example-driven CLI help and structured agent context output.

Changes:
- Top-level, init, config, and agent-context help now include copy-pasteable examples.
- --json is available globally for machine-readable config/agent outputs and compact extraction JSON.
- Added invo agent-context --json with command contracts, examples, config path/existence, active keys, and parsed-only keys.

Tests:
- cargo test
- cargo build
- manual --help and agent-context JSON smoke checks
<!-- SECTION:FINAL_SUMMARY:END -->
