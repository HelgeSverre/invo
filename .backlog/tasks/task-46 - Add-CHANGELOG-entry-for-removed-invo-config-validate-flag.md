---
id: TASK-46
title: Add CHANGELOG entry for removed invo config --validate flag
status: Done
assignee: []
created_date: '2026-05-27 12:00'
updated_date: '2026-05-27 12:00'
labels:
  - docs
  - changelog
dependencies:
  - TASK-8
references:
  - REVIEW-CLI.md
modified_files:
  - CHANGELOG.md
  - README.md
priority: medium
ordinal: 46000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
TASK-8 replaces `invo config --validate` with nested subcommands
(`invo config validate`, `show`, `path`, `doctor`, `schema`). Scripts using
the old form get a Clap parse error with no migration hint. Document the
removal in a CHANGELOG and refresh living docs to use the new form only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 CHANGELOG documents the old to new mapping clearly.
- [ ] #2 Optional: accept the legacy `--validate` flag as a deprecated alias that emits a stderr warning and dispatches to `config validate` for one release. (skipped — documented only)
- [x] #3 README and any REVIEW-CLI / usage examples reference the new subcommand form only.
<!-- AC:END -->

## Notes

- REVIEW-CLI.md is kept as a historical record of the pre-TASK-8 state and
  was not rewritten.
- Optional alias (AC #2) skipped because TASK-8 itself is still pending on
  this branch; adding a deprecated alias before the new subcommands land
  would be premature and would require non-trivial argument-parsing changes.
