---
id: TASK-12
title: Make identity config fully active and discoverable
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
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Buyer identity should be a clear optional feature that helps distinguish the user/buyer from the vendor.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 All active identity fields are included in the extraction prompt or removed from config.
- [ ] #2 address and vat_number behavior is fixed or documented as unsupported.
- [ ] #3 README explains when to add identity context.
<!-- AC:END -->
