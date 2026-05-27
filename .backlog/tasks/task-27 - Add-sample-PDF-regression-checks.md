---
id: TASK-27
title: Add sample PDF regression checks
status: Done
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 00:00'
labels:
  - tests
  - pdf
dependencies: []
modified_files:
  - src/pdf.rs
  - src/extractor.rs
  - tests/cli_output.rs
priority: medium
ordinal: 29000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use the existing sample PDFs to protect PDF text extraction, metadata, hash/page-count behavior, and output shape without requiring live LLM calls by default.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 At least a few checked-in sample PDFs are used in automated tests.
- [x] #2 Tests do not require a live LLM unless explicitly opt-in.
- [x] #3 PDF text extraction, short hash, and page-count behavior are covered.
- [x] #4 Expected output wrapper shape remains stable.
<!-- AC:END -->
