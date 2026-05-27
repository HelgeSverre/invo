---
id: TASK-71
title: Make short_sha256 robust to changes in hash representation
status: Done
assignee: []
created_date: '2026-05-27 00:12'
updated_date: '2026-05-27 00:56'
labels:
  - code-review
  - reliability
  - pdf
dependencies:
  - TASK-32
references:
  - src/pdf.rs
priority: low
ordinal: 73000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/pdf.rs:50-55 does hash[..16].to_string() on a hex-formatted hash. Safe today because hex is ASCII. Any refactor that introduces non-ASCII (base64 padding, locale-aware formatting) panics at a non-char-boundary. Also, 16 hex chars = 64 bits — borderline for future dedup use (TASK-32) given birthday collisions. Reviewer: adversarial.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Slice operates by character count (chars().take(16)) or asserts the hex invariant explicitly
- [ ] #2 Bit width is documented and revisited when TASK-32 lands
- [ ] #3 Test demonstrates behavior is unchanged for the current hex output
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Switched short_sha256 from byte-index slicing to chars().take(SHORT_HASH_LEN).collect() so a future non-ASCII encoding change can't panic on char boundaries. Added regression tests asserting 16 lowercase hex chars and determinism. Documented the 64-bit width tradeoff for future TASK-32 dedup work in the function comment.
<!-- SECTION:NOTES:END -->
