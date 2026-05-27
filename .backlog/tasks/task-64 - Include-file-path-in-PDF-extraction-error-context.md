---
id: TASK-64
title: Include file path in PDF extraction error context
status: Done
assignee: []
created_date: '2026-05-27 00:11'
updated_date: '2026-05-27 00:43'
labels:
  - code-review
  - reliability
  - bug
dependencies: []
references:
  - src/extractor.rs
modified_files:
  - src/extractor.rs
priority: low
ordinal: 66000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/extractor.rs:53-62 wraps PDF extraction with .context("Failed to extract text from PDF") without interpolating the input file path. In single-file error-propagation mode the anyhow chain lacks an independently useful path, hurting both human and log-scraper readability. Reviewer: reliability.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 PDF extraction error context includes the input file path via .with_context(|| format!(...))
- [x] #2 Existing tests continue to pass
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Resolved during the TASK-24 refactor. PDF extraction errors now use with_context and include the input path, and the test suite passes.
<!-- SECTION:FINAL_SUMMARY:END -->
