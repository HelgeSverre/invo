---
id: TASK-28
title: Add OCR fallback for scanned PDFs
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
labels:
  - feature
  - pdf
  - ocr
dependencies: []
modified_files:
  - src/pdf.rs
  - src/config.rs
  - README.md
priority: medium
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement an OCR path for image-only or scanned PDFs so extraction can proceed when native PDF text extraction produces no useful text.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Image-only or scanned PDFs can produce extracted text through OCR when OCR is enabled.
- [ ] #2 The output extraction.ocr_used field is true when OCR is used.
- [ ] #3 Users get actionable errors when OCR is unavailable or disabled.
- [ ] #4 README documents OCR requirements and limitations.
- [ ] #5 Tests cover OCR-disabled and OCR-unavailable behavior without requiring system OCR in default test runs.
<!-- AC:END -->
