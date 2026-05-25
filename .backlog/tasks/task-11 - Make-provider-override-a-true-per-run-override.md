---
id: TASK-11
title: Make provider override a true per-run override
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 14:43'
labels:
  - llm
  - cli
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/llm.rs
  - README.md
priority: medium
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provider override should work without requiring that provider to already be enabled, or the flag/docs should be renamed to match actual behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 --provider openai can select an OpenAI provider entry even if disabled, or docs/flag name are changed to match actual behavior.
- [x] #2 Errors tell the user exactly what config or environment value is missing.
- [x] #3 README examples match the implemented semantics.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Change provider override selection to pick a configured provider by name even if disabled. 2. Improve missing provider/API-key error messages with exact config or environment requirements. 3. Update tests and README examples to match the semantics.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Provider override now selects a configured provider by name even when that config entry is disabled.

Improved missing provider and OpenAI API key messages with exact config/environment requirements.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Made provider override a true per-run selector for configured providers.

Changes:
- --provider now selects a matching provider entry by name regardless of enabled state.
- Missing provider and OpenAI api_key_env/env-var errors now explain the exact missing config or environment value.
- README provider examples and semantics were updated.

Tests:
- cargo test
<!-- SECTION:FINAL_SUMMARY:END -->
