---
id: TASK-13
title: Make vendor config fields active or remove them
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - config
  - extraction
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/config/schema.rs
  - src/extraction/prompt.rs
  - README.md
priority: medium
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Vendor config should not parse fields that do nothing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 currency and hints affect extraction, or they are removed from active config/docs.
- [ ] #2 Known vendor examples only show active fields.
- [ ] #3 Config doctor can identify inactive vendor fields if any remain.
<!-- AC:END -->
