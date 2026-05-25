---
id: TASK-15
title: Support pipeline-friendly file input
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 14:59'
labels:
  - cli
  - automation
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - src/main.rs
  - src/pdf.rs
  - src/extractor.rs
  - tests/cli_output.rs
priority: low
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add stdin and file-list input options for scripts and agents.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 invo - accepts PDF bytes from stdin or returns an actionable unsupported error.
- [x] #2 --files-from <path> supports newline-delimited file lists.
- [x] #3 Metadata for stdin input is deterministic.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a CLI --files-from option that merges newline-delimited paths with positional file/glob arguments. 2. Add a deterministic input abstraction so '-' reads PDF bytes from stdin with stable metadata. 3. Extend PDF extraction to work from bytes and add unit/integration coverage for stdin, file lists, and metadata.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added '-' stdin input handling with PDF magic validation and deterministic stdin.pdf file metadata.

Added --files-from for newline-delimited paths/globs plus unit and integration coverage for file-list and stdin error behavior.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added pipeline-friendly input options for scripts and agents.

Changes:
- Added --files-from <path> for newline-delimited file/glob lists, with --files-from - reserved for reading the list from stdin.
- Added '-' as PDF stdin input with actionable errors for empty/non-PDF input.
- Introduced an InvoiceInput abstraction so stdin uses deterministic file metadata (stdin.pdf) while regular files keep path/name metadata.
- Extended PDF extraction to operate from bytes as well as paths.

Tests:
- cargo test
<!-- SECTION:FINAL_SUMMARY:END -->
