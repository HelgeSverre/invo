---
id: TASK-19
title: Make Rust quality checks green and add CI
status: Done
assignee:
  - '@helge'
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 17:22'
labels:
  - quality
  - ci
dependencies: []
modified_files:
  - src/main.rs
  - .github/workflows/ci.yml
  - Cargo.toml
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add automated Rust quality checks and make the current local quality commands pass consistently.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 cargo fmt --check passes.
- [x] #2 cargo test passes.
- [x] #3 cargo clippy --all-targets -- -D warnings passes.
- [x] #4 CI runs formatting, tests, and clippy on push or pull request.
- [x] #5 The documented Rust version/MSRV is either tested in CI or corrected in README/Cargo metadata.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Fix clippy warning in src/main.rs (replace .clone() with from_ref)
2. Verify cargo fmt --check passes
3. Verify cargo test passes
4. Verify cargo clippy --all-targets -- -D warnings passes
5. Create .github/workflows/ci.yml with Rust quality checks
6. Update README.md to document MSRV (Rust 1.70+)
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
1. Fixed clippy warning in src/main.rs line 790: replaced path.clone() with std::slice::from_ref(&path)
2. Verified cargo fmt --check passes
3. Verified cargo test passes (24 unit tests + 12 integration tests)
4. Verified cargo clippy --all-targets -- -D warnings passes
5. Created .github/workflows/ci.yml with fmt, test, and clippy checks
6. Added rust-version = "1.70" to Cargo.toml to document MSRV
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented Rust quality checks and CI for invo project.

Changes:
- Fixed clippy warning in src/main.rs: replaced path.clone() with std::slice::from_ref(&path) in expand_inputs_accepts_pdf_paths test
- Created .github/workflows/ci.yml that runs cargo fmt --check, cargo test, and cargo clippy --all-targets -- -D warnings on push and pull request
- Added rust-version = "1.70" to Cargo.toml to document MSRV (matches README requirement)

Quality checks now pass:
- cargo fmt --check: passes
- cargo test: 36 tests pass (24 unit + 12 integration)
- cargo clippy --all-targets -- -D warnings: passes with no warnings

The CI workflow will automatically enforce these checks on all future pushes and pull requests.
<!-- SECTION:FINAL_SUMMARY:END -->
