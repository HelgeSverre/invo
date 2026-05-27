---
id: TASK-82
title: Friendlier migration error for old action YAML shape
status: Done
assignee: []
created_date: '2026-05-27 16:04'
updated_date: '2026-05-27 16:07'
labels:
  - config
  - task-77-followup
  - user-experience
dependencies: []
references:
  - src/config.rs
  - CHANGELOG.md
priority: low
ordinal: 84000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
TASK-77 broke the old action YAML schema. The pre-TASK-77 shape was:

  actions:
    - type: copy
      destination: archive/

The post-TASK-77 shape is:

  actions:
    - type: file
      kind: copy
      destination: archive/

Strict validation now rejects the old shape with serde's default message: 'unknown variant `copy`, expected one of `file`, `export`, `hook`'. This is technically correct but doesn't hint at the actual migration path.

Add either:
1. A serde alias on Action::File that accepts 'copy' and 'move' as deprecated aliases, then emits a doctor warning that they should be updated to the new shape. (Soft migration.)
2. A custom deserialize that recognizes the old tags specifically and produces an error message pointing at TASK-77/CHANGELOG. (Hard migration with hint.)

The user direction in TASK-77 was 'no shim, no backwards compat'. Option 2 is consistent with that — keep the rejection but make it actionable.

Surface: src/config.rs Action enum.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Submitting actions with type: copy or type: move produces an error message mentioning the new type: file + kind: copy/move shape
- [x] #2 Error message includes a reference to the CHANGELOG entry or TASK-77 for full migration notes
- [x] #3 Unit test covers the migration-pointing error path
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
parse_config_str now detects the pre-TASK-77 'unknown variant copy' / 'unknown variant move' patterns and appends an actionable migration hint pointing at the new type:file + kind:copy/move shape and the CHANGELOG. Three unit tests cover the two old variants and a third sanity test (an unrelated 'unknown variant' like 'type: archive' must NOT show the misleading TASK-77 hint). DoD #4 (docs) and #5 (notes) not applicable — error message change only; user-facing CHANGELOG already documents the migration. Test count: 130+16 -> 133+16.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied and checked
- [x] #2 Relevant tests are added or updated
- [x] #3 cargo fmt, cargo test, and cargo clippy --all-targets -- -D warnings pass
- [ ] #4 Documentation/help/config examples are updated when behavior or user-facing output changes
- [ ] #5 Backlog task notes and final summary describe what changed and how it was verified
- [x] #6 Self-review confirms no unrelated changes, secrets, or debug artifacts are included
<!-- DOD:END -->
