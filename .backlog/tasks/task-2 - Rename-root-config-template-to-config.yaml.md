---
id: TASK-2
title: Rename root config template to config.yaml
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - config
  - docs
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - default_config.yaml
  - src/main.rs
  - README.md
priority: medium
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Rename the root config template from default_config.yaml to config.yaml and update runtime/docs references.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 invo init embeds and writes the renamed root template.
- [ ] #2 README and review/docs links no longer reference default_config.yaml.
- [ ] #3 No stale default_config names remain except intentionally historical notes.
<!-- AC:END -->
