---
id: TASK-19
title: Make Rust quality checks green and add CI
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
labels:
  - quality
  - ci
dependencies: []
modified_files:
  - src/main.rs
  - .github/workflows/ci.yml
  - README.md
priority: high
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add automated Rust quality checks and make the current local quality commands pass consistently.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 cargo fmt --check passes.
- [ ] #2 cargo test passes.
- [ ] #3 cargo clippy --all-targets -- -D warnings passes.
- [ ] #4 CI runs formatting, tests, and clippy on push or pull request.
- [ ] #5 The documented Rust version/MSRV is either tested in CI or corrected in README/Cargo metadata.
<!-- AC:END -->
