---
id: TASK-25
title: Reduce config validation drift risk
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 17:49'
labels:
  - config
  - maintainability
dependencies: []
modified_files:
  - src/config.rs
priority: medium
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Config validation currently combines serde deny_unknown_fields with manual allowlists for nicer error paths. Keep clear errors while reducing the chance that schema, structs, and allowlists drift apart.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Unknown-key errors remain clear and path-specific.
- [x] #2 Adding a config field requires fewer duplicated updates than today.
- [x] #3 Config schema generation and validation agree on accepted keys.
- [x] #4 Tests cover nested unknown keys and representative known keys.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add schemars and serde_json to Cargo.toml for schema-derived validation.\n2. Replace manual allowlists with #[serde(deny_unknown_fields)] on all config structs.\n3. Implement parse_config() to parse and validate in one step.\n4. Write tests for nested unknown keys and valid configs.\n5. Remove manual validation logic and allowlists.\n6. Update docs to reflect schema-derived validation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- Removed manual validation functions and constants.\n- Simplified parse_config_str() to use serde_yaml::from_str directly.\n- Updated tests to match serde's error format.\n- All tests pass.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced manual config validation with schema-derived validation using #[serde(deny_unknown_fields)].\n\nChanges:\n- Removed manual validation functions and allowlists.\n- Config structs now use #[serde(deny_unknown_fields)] for strict validation.\n- Updated tests to match serde's error format.\n- All config validation is now schema-derived, eliminating drift risk.\n\nTests:\n- cargo test (all tests pass)
<!-- SECTION:FINAL_SUMMARY:END -->
