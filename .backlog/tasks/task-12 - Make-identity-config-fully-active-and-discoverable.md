---
id: TASK-12
title: Make identity config fully active and discoverable
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
  - src/config.rs
  - src/prompt.rs
  - README.md
  - config.yaml
priority: medium
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Buyer identity should be a clear optional feature that helps distinguish the user/buyer from the vendor.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All active identity fields are included in the extraction prompt or removed from config.
- [x] #2 address and vat_number behavior is fixed or documented as unsupported.
- [x] #3 README explains when to add identity context.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Include all identity fields in is_configured and the extraction prompt. 2. Document address and VAT behavior as active buyer context. 3. Add tests/README guidance for when to add identity context.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Identity address and VAT number now activate identity context and are included in the extraction prompt.

README and config.yaml now document identity as optional buyer context.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Made identity config fully active and documented as buyer context.

Changes:
- identity.address and identity.vat_number now count as configured identity fields.
- Extraction prompts include address and VAT number alongside company/name/email/aliases.
- README explains when to add identity context, and config.yaml shows active commented fields.

Tests:
- cargo test
<!-- SECTION:FINAL_SUMMARY:END -->
