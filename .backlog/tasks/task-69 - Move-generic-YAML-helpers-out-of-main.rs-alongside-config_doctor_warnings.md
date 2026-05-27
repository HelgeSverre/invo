---
id: TASK-69
title: Move generic YAML helpers out of main.rs alongside config_doctor_warnings
status: Done
assignee: []
created_date: '2026-05-27 00:11'
updated_date: '2026-05-27 00:43'
labels:
  - code-review
  - maintainability
  - refactor
dependencies: []
references:
  - src/main.rs
modified_files:
  - src/config_commands.rs
  - src/main.rs
priority: low
ordinal: 71000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs:472-493 defines value_at, mapping_get, and value_is_empty — pure serde_yaml::Value traversal utilities with no dependency on CLI concerns. They live in main.rs only because config_doctor_warnings uses them. Should travel together when config_doctor_warnings moves. Reviewer: maintainability.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 value_at / mapping_get / value_is_empty move alongside config_doctor_warnings (see task for that move)
- [x] #2 main.rs imports no serde_yaml::{Value, Mapping}
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Resolved by TASK-24. The YAML helper functions moved with config_doctor_warnings into src/config_commands.rs, and main.rs no longer imports serde_yaml::Value or Mapping.
<!-- SECTION:FINAL_SUMMARY:END -->
