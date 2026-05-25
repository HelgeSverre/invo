---
id: TASK-31
title: Implement post-processing actions
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
labels:
  - feature
  - workflow
dependencies: []
modified_files:
  - src/config.rs
  - src/main.rs
  - README.md
priority: low
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Config already models copy, move, export, hook, and notify actions, but they are parsed-only. Implement or deliberately narrow this post-processing surface.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Configured copy and move actions run after successful extraction.
- [ ] #2 Configured export actions write the selected output format to the configured destination.
- [ ] #3 Hook actions execute with documented arguments/environment and honor on_error.
- [ ] #4 Notify actions are either implemented or removed from accepted config.
- [ ] #5 config doctor, README, and agent-context no longer report implemented actions as parsed-only.
<!-- AC:END -->
