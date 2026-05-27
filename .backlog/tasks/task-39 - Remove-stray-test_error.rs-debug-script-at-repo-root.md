---
id: TASK-39
title: Remove stray test_error.rs debug script at repo root
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:07'
updated_date: '2026-05-27 00:45'
labels:
  - code-review
  - chore
dependencies: []
references:
  - test_error.rs
priority: high
ordinal: 41000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
test_error.rs is a 7-line debug script at repo root with fn main() calling invo::config::parse_config_str. The crate has no [lib] in Cargo.toml so the import path does not exist. Cargo ignores loose .rs files at repo root so it never compiles, but it is confusing for any future contributor and references a non-existent library API. Reviewers: correctness, maintainability, adversarial, agent-native, security.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 test_error.rs is deleted from the repository
- [x] #2 The functionality it was verifying (parse_config_str rejecting unknown top-level keys) is covered by an existing unit test in src/config.rs
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Confirm parse_config_str unknown-key behavior is covered by an existing unit test in src/config.rs
2. Delete test_error.rs
3. Verify cargo check still passes
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Existing test rejects_unknown_top_level_config_key_with_path (src/config.rs:526) covers the identical scenario — parsing identitty: {} and asserting the error message contains the typo — so the behavior is fully verified without test_error.rs.

Final Summary: Deleted test_error.rs (7-line dead debug script at repo root). The file used invo::config::parse_config_str to test that a typo in a top-level config key (identitty) produces an error, but the crate has no [lib] so it never compiled. The identical scenario is fully covered by the unit test rejects_unknown_top_level_config_key_with_path in src/config.rs (line 526), which passes the same YAML string and asserts the error message includes the typo. cargo check confirms no other file referenced test_error.rs; the two pre-existing compile errors in src/extractor.rs are unrelated to this change.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Deleted test_error.rs, a dead root-level debug script that Cargo ignored and that referenced a non-existent library import path. The unknown top-level config-key behavior it attempted to exercise is already covered by src/config.rs::tests::rejects_unknown_top_level_config_key_with_path. Current validation passes with cargo test and cargo clippy --all-targets -- -D warnings.
<!-- SECTION:FINAL_SUMMARY:END -->
