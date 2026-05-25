---
id: TASK-10
title: Make global flags usable with subcommands
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 14:43'
labels:
  - cli
  - usability
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
  - README.md
  - tests/cli_output.rs
priority: medium
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reduce CLI friction around config flag placement for subcommands.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 --config works before or after applicable subcommands, or help clearly documents the accepted placement.
- [x] #2 Validation examples use the supported placement.
- [x] #3 Clap tests or snapshots cover the chosen behavior.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Mark --config as a global Clap option so it works before or after subcommands. 2. Update validation examples to the supported placement. 3. Add CLI regression tests for post-subcommand --config.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Marked --config as a global Clap option so it works before or after applicable subcommands.

Updated examples and integration tests to cover post-subcommand config placement.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Made --config placement more forgiving for subcommands.

Changes:
- --config is now a global Clap option and works before or after config/init subcommands.
- Config validation help and README examples show supported placement.
- Added CLI regression tests for invo config validate -c config.yaml.

Tests:
- cargo test
- manual init/validate --config placement checks
<!-- SECTION:FINAL_SUMMARY:END -->
