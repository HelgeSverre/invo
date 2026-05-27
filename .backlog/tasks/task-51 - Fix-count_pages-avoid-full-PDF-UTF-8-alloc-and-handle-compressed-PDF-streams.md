---
id: TASK-51
title: 'Fix count_pages: avoid full-PDF UTF-8 alloc and handle compressed PDF streams'
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:09'
labels:
  - code-review
  - performance
  - pdf
  - bug
dependencies: []
references:
  - src/pdf.rs
priority: medium
ordinal: 53000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/pdf.rs:58-63 calls String::from_utf8_lossy on the entire binary PDF byte slice (~3x size in heap with U+FFFD) to substring-search '/Type /Page'. Multi-MB PDFs allocate transient strings of comparable size. Secondary correctness issue: '/Type /Page' does not appear inside compressed/encrypted PDF streams (PDF 1.5+), so count_pages silently returns 1 for many real PDFs. Cosmetic today (only logged) but becomes a reliability concern once any code uses page count for truncation or routing. Reviewers: performance, adversarial, correctness.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Allocation: count_pages searches bytes directly (e.g. bytes.windows(...).count()) without allocating a String copy
- [x] #2 Correctness: compressed-stream PDFs report a real page count (parse via lopdf or have pdf-extract report it)
- [x] #3 Pattern match avoids false positives on '/Type /Pages' (use a trailing delimiter)
- [x] #4 Regression tests cover an uncompressed multi-page PDF, a compressed multi-page PDF, and a single-page PDF
<!-- AC:END -->
