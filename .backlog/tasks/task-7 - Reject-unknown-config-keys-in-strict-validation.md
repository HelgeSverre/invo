---
id: TASK-7
title: Reject unknown config keys in strict validation
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 14:43'
labels:
  - config
  - validation
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/config.rs
  - config.yaml
  - tests/cli_output.rs
priority: medium
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prevent typos such as identitty from silently passing config validation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Strict validation reports unknown top-level and nested keys.
- [x] #2 Error messages include the invalid key path.
- [x] #3 The active config template passes strict validation.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add strict unknown-key validation for the active YAML config schema, including nested key paths. 2. Wire strict parsing into config loading and validation. 3. Add tests for unknown top-level/nested keys and the active config template.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added strict config key validation before deserialization, including explicit dotted paths for nested unknown keys.

Added unit and CLI tests for unknown top-level/nested keys and the active config template.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added strict config validation so typos no longer pass silently.

Changes:
- Added deny-unknown-field coverage and explicit YAML key-path validation for config.yaml.
- Unknown keys now fail with messages such as identity.vatnumbber or identitty.
- Verified the active config.yaml template passes strict parsing.

Tests:
- cargo test
- manual unknown-key config validate smoke check
<!-- SECTION:FINAL_SUMMARY:END -->
