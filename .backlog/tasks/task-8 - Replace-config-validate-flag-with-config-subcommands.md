---
id: TASK-8
title: Replace config validate flag with config subcommands
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
  - src/main.rs
  - README.md
  - tests/cli_output.rs
priority: medium
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make config management discoverable and avoid a silent no-op invo config command.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 invo config no longer exits silently.
- [x] #2 Config path, show --effective, validate, doctor, and schema behavior exists or any deferred subcommand is explicitly documented.
- [x] #3 Help output includes copy-pasteable examples.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the config --validate boolean with config subcommands. 2. Implement path, show --effective, validate, doctor, and schema behavior. 3. Ensure bare invo config displays actionable help instead of silently succeeding.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced the old config --validate boolean with path/show/validate/doctor/schema subcommands.

Bare invo config now shows actionable Clap help instead of exiting successfully with no output.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reworked config management into discoverable subcommands.

Changes:
- Added invo config path, show --effective, validate, doctor, and schema.
- Removed the silent no-op behavior for bare invo config.
- Added copy-pasteable config help examples and integration coverage.

Tests:
- cargo test
- cargo build
- manual config help/path/show/validate/doctor/schema smoke checks
<!-- SECTION:FINAL_SUMMARY:END -->
