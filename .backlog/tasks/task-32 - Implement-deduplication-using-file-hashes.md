---
id: TASK-32
title: Implement deduplication using file hashes
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 00:44'
labels:
  - feature
  - deduplication
dependencies: []
modified_files:
  - src/dedup.rs
  - src/config.rs
  - src/main.rs
  - src/extractor.rs
  - src/pdf.rs
  - src/config_commands.rs
  - src/agent_context.rs
  - README.md
priority: medium
ordinal: 33000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Config includes processing.deduplication settings, and extraction already computes short file hashes. Implement hash-based duplicate detection or remove the parsed-only config surface.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 When deduplication is enabled, previously processed file hashes can be skipped or reported according to documented behavior.
- [ ] #2 The hash store location is configurable.
- [ ] #3 Batch output remains parseable when duplicates are skipped.
- [ ] #4 README documents deduplication behavior and hash-store format.
- [ ] #5 Tests cover first-seen and duplicate file behavior.
<!-- AC:END -->
