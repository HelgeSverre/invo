---
id: TASK-66
title: Update rust-version to match the actual MSRV
status: Done
assignee: []
created_date: '2026-05-27 00:11'
updated_date: '2026-05-27 04:00'
labels:
  - code-review
  - docs
  - ci
dependencies: []
references:
  - Cargo.toml
  - README.md
priority: low
ordinal: 68000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Cargo.toml:8 declares rust-version = "1.70", but rig-core 0.37 requires edition 2024 (Rust 1.85+). The transitive dep tree no longer compiles on 1.70. README mirrors the stale '1.70+' claim. Reviewer: correctness.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo.toml rust-version reflects the actual MSRV (verify via cargo msrv or local toolchain test)
- [x] #2 README 'Requires Rust 1.70+' text is updated
- [x] #3 CI uses the same MSRV in the toolchain step
<!-- AC:END -->



## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Bumped Cargo.toml rust-version 1.70 -> 1.85 (rig-core 0.37 requires edition 2024). Updated README install requirement. Pinned CI toolchain to 1.85 so MSRV regressions surface in CI rather than at user install time.
<!-- SECTION:NOTES:END -->
