---
id: TASK-21
title: Remove unused direct dependencies
status: Done
assignee:
  - '@agent'
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 17:28'
labels:
  - cleanup
  - dependencies
dependencies: []
modified_files:
  - Cargo.toml
  - Cargo.lock
priority: medium
ordinal: 24000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Review direct dependencies and remove crates that are not used by current runtime or tests, such as future-use dependencies that can be re-added when needed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Unused direct dependencies are removed from Cargo.toml.
- [x] #2 Cargo.lock is updated accordingly.
- [x] #3 cargo test passes.
- [x] #4 cargo clippy --all-targets -- -D warnings passes.
- [x] #5 Dependencies retained in Cargo.toml are used by current code or tests.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review current Cargo.toml dependencies
2. Identify unused direct dependencies using cargo tree and cargo udig
3. Remove unused dependencies from Cargo.toml
4. Update Cargo.lock
5. Run cargo test to ensure nothing breaks
6. Run cargo clippy to ensure no new warnings
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Removed unused direct dependencies (rayon and minijinja) from Cargo.toml. Updated Cargo.lock accordingly. All tests pass and cargo clippy shows no warnings. The remaining dependencies are all actively used by the codebase.
<!-- SECTION:FINAL_SUMMARY:END -->
