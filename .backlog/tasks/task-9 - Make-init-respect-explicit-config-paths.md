---
id: TASK-9
title: Make init respect explicit config paths
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
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make --config <path> control where config initialization writes, or remove the misleading global interaction.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 invo -c /tmp/invo.yaml init writes /tmp/invo.yaml.
- [x] #2 Existing files are not overwritten unless an explicit force flag is used.
- [x] #3 Help documents the init destination behavior.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Make init use the explicit --config path when provided. 2. Keep no-overwrite behavior and add an explicit --force flag for replacement. 3. Add help/docs and CLI tests for destination behavior.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Changed init to write to the explicit --config path when present and added --force for intentional overwrites.

Covered init destination and overwrite behavior in CLI tests and help docs.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Made init respect explicit config paths and safer overwrite semantics.

Changes:
- invo -c /tmp/invo.yaml init now writes that exact path.
- Existing config files are preserved unless --force is supplied.
- Help and README now document init destination behavior.

Tests:
- cargo test
- manual init before/after --config placement smoke checks
<!-- SECTION:FINAL_SUMMARY:END -->
