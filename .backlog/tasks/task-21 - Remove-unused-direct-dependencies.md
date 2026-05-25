---
id: TASK-21
title: Remove unused direct dependencies
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
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
- [ ] #1 Unused direct dependencies are removed from Cargo.toml.
- [ ] #2 Cargo.lock is updated accordingly.
- [ ] #3 cargo test passes.
- [ ] #4 cargo clippy --all-targets -- -D warnings passes.
- [ ] #5 Dependencies retained in Cargo.toml are used by current code or tests.
<!-- AC:END -->
