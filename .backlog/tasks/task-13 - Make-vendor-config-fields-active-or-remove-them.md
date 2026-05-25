---
id: TASK-13
title: Make vendor config fields active or remove them
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 14:43'
labels:
  - config
  - extraction
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/prompt.rs
  - src/main.rs
  - README.md
  - config.yaml
priority: medium
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Vendor config should not parse fields that do nothing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 currency and hints affect extraction, or they are removed from active config/docs.
- [x] #2 Known vendor examples only show active fields.
- [x] #3 Config doctor can identify inactive vendor fields if any remain.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Make vendor currency and hints active by adding them to the extraction prompt. 2. Update known vendor examples and config docs to show only active fields. 3. Ensure config doctor covers inactive parsed-only fields elsewhere.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Vendor currency and hints are now included in known-vendor prompt context.

Config docs/examples show vendor fields that are active; config doctor reports other parsed-only fields.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Made vendor config fields active and kept config doctor available for inactive parsed-only settings.

Changes:
- vendors[].currency and vendors[].hints now affect extraction through the known-vendor prompt.
- README and config.yaml examples now show active vendor fields only.
- Config doctor reports remaining parsed-only non-vendor settings when users configure them.

Tests:
- cargo test
- manual config doctor smoke check
<!-- SECTION:FINAL_SUMMARY:END -->
