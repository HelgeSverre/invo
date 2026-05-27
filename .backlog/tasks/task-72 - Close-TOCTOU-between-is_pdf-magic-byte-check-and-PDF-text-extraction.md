---
id: TASK-72
title: Close TOCTOU between is_pdf magic-byte check and PDF text extraction
status: Done
assignee: []
created_date: '2026-05-27 00:12'
updated_date: '2026-05-27 00:59'
labels:
  - code-review
  - security
  - pdf
dependencies: []
references:
  - src/input.rs
  - src/extractor.rs
modified_files:
  - src/input.rs
  - src/extractor.rs
priority: low
ordinal: 74000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/input.rs validates candidate file paths by opening and reading the PDF magic bytes, then later InvoiceInputSpec::read opens and reads the path again before extraction. An attacker who can swap the target between these calls (symlink swap, antivirus rewrite, cloud-sync replacement) can feed different content than the file that passed validation. The old is_pdf function is gone, but the time-of-check/time-of-use boundary remains in input materialization.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Path validation and byte materialization operate on the same opened file handle or on one already-read byte buffer.
- [ ] #2 If the race is accepted, the trust boundary and residual risk are documented explicitly.
- [ ] #3 Regression test pins the chosen behavior.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced two-open input flow with single read_pdf_if_valid() at validation time. InvoiceInputSpec now carries Vec<u8> directly (removed the Path | Bytes source enum), so InvoiceInputSpec::read() simply hands the captured buffer through. Symlink swap / sync rewrite between validation and extraction can no longer substitute different content. Added expand_inputs_captures_bytes_at_validation_time regression test that mutates the file after expand_inputs returns and asserts the original bytes survive.
<!-- SECTION:NOTES:END -->
