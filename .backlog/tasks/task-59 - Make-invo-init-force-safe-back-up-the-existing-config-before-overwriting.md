---
id: TASK-59
title: 'Make ''invo init --force'' safe: back up the existing config before overwriting'
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:10'
labels:
  - code-review
  - bug
  - cli
  - config
dependencies: []
references:
  - src/main.rs
priority: medium
ordinal: 61000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs:215 writes include_str!("../config.yaml") over the existing config when --force is passed. A user with a customized ~/.config/invo/config.yaml (API keys, vendor patterns, identity) who runs 'invo init --force' (perhaps from a docs snippet) loses everything. No timestamped backup, no per-field merge, no warning showing the destination path before overwrite. Reviewer: adversarial.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 When --force overwrites an existing config, the original file is preserved as <path>.bak.YYYYMMDDHHMMSS before the write
- [x] #2 The backup path is logged to stderr on success
- [x] #3 Integration test asserts the backup file exists after a forced overwrite
<!-- AC:END -->
