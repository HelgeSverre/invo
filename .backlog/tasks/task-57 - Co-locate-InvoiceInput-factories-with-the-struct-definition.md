---
id: TASK-57
title: Co-locate InvoiceInput factories with the struct definition
status: Done
assignee: []
created_date: '2026-05-27 00:10'
updated_date: '2026-05-27 00:43'
labels:
  - code-review
  - maintainability
  - refactor
dependencies:
  - TASK-24
references:
  - src/main.rs
  - src/extractor.rs
modified_files:
  - src/input.rs
  - src/main.rs
  - src/extractor.rs
priority: medium
ordinal: 59000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
InvoiceInput is defined in src/extractor.rs but its factory functions (collect_inputs, expand_inputs, read_files_from, parse_file_list, read_stdin_pdf_input, is_stdin_marker — ~90 lines) live in src/main.rs. Construction logic is split across two files. Lifting these into a dedicated input module shrinks main.rs and co-locates the InvoiceInput surface. Aligns with TASK-24. Reviewer: maintainability.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 InvoiceInput factory functions move into src/extractor.rs or a new src/input.rs
- [x] #2 main.rs no longer contains InvoiceInput construction logic
- [x] #3 Existing tests still pass without modification
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Resolved by TASK-24. InvoiceInput and input collection/factory logic now live in src/input.rs; main.rs no longer contains PDF input construction logic and tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
