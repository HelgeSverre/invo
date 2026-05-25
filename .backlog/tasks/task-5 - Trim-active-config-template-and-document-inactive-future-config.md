---
id: TASK-5
title: Trim active config template and document inactive future config
status: To Do
assignee: []
created_date: '2026-05-25 12:57'
labels:
  - config
  - ux
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - config.yaml
  - README.md
priority: high
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Keep the generated config.yaml honest by including only active runtime keys and moving future settings out of the default template.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Parsed-but-unused sections are removed from generated config or moved to clearly marked future/example docs.
- [ ] #2 OCR, retry, action, parallelism, dedupe, and logging contradictions are removed from the default template.
- [ ] #3 Config docs include a status table for active versus future keys.
<!-- AC:END -->
