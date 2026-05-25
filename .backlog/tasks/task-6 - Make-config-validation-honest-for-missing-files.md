---
id: TASK-6
title: Make config validation honest for missing files
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 14:43'
labels:
  - config
  - cli
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/config.rs
  - src/main.rs
  - tests/cli_output.rs
priority: medium
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
config validate must distinguish an existing valid config file from a missing config path using built-in defaults.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Missing config path reports missing status and effective defaults.
- [x] #2 Existing config path reports the path validated.
- [x] #3 Machine-readable validation output is available or intentionally deferred consistently with config command design.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add config loading metadata so validation can report path existence separately from effective defaults. 2. Update config validation output for missing and existing paths. 3. Add CLI tests for missing/default and existing config validation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented load_config_status so validation reports whether config came from disk or built-in defaults.

Added human and JSON validation output plus CLI regression tests for missing and existing config paths.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Made config validation honest about missing files while keeping effective defaults usable.

Changes:
- Added config loading metadata that tracks path and existence separately from the effective Config.
- Updated invo config validate to report missing/default status, existing validated paths, and JSON validation output.
- Added CLI tests for missing config homes, existing config paths, and machine-readable validation.

Tests:
- cargo test
- cargo build
- manual config validate JSON/missing-file smoke checks
<!-- SECTION:FINAL_SUMMARY:END -->
