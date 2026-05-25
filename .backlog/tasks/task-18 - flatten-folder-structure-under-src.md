---
id: TASK-18
title: Flatten Rust module layout under src/
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 13:53'
updated_date: '2026-05-25 13:57'
labels: []
dependencies: []
modified_files:
  - src/config.rs
  - src/invoice.rs
  - src/prompt.rs
  - src/llm.rs
  - src/pdf.rs
  - src/extractor.rs
  - src/main.rs
  - src/config/mod.rs
  - src/config/schema.rs
  - src/extraction/mod.rs
  - src/extraction/prompt.rs
  - src/extraction/schema.rs
  - src/llm/mod.rs
  - src/pdf/mod.rs
  - src/pipeline.rs
priority: medium
ordinal: 18000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current nested mod.rs-based source layout with a flat src/ module layout until the codebase has enough complexity to justify folders. Target layout: config.rs for configuration, invoice.rs for extraction result/domain structs, prompt.rs for extraction prompts, llm.rs for LLM provider calls, pdf.rs for PDF text extraction, extractor.rs for end-to-end invoice extraction orchestration, and main.rs for CLI wiring.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Source files use a flat src/ layout with no src/*/mod.rs modules.
- [x] #2 Extraction schema types are exposed from invoice.rs and prompt builders are exposed from prompt.rs without ambiguous generic schema.rs filenames.
- [x] #3 pipeline.rs is renamed to extractor.rs and main.rs imports extract_invoice from extractor.
- [x] #4 Existing tests pass after the module layout change.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Move nested module contents into flat src/*.rs files with clear domain names. 2. Update module declarations and crate:: imports in main/extractor/llm/prompt/tests. 3. Remove now-empty nested module folders and old pipeline.rs. 4. Run formatting and tests, then check acceptance criteria.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Flattened nested Rust modules into src/config.rs, src/invoice.rs, src/prompt.rs, src/llm.rs, src/pdf.rs, and src/extractor.rs.

Updated main.rs and internal imports to use extractor/invoice/prompt instead of extraction/pipeline modules.

Ran cargo fmt and cargo test successfully: 15 tests passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Flattened the Rust source module layout under src/ and removed the current mod.rs folder structure.

Changes:
- Collapsed config loading and config schema into src/config.rs.
- Moved invoice extraction/result structs into src/invoice.rs and prompt builders into src/prompt.rs.
- Moved LLM and PDF modules to src/llm.rs and src/pdf.rs.
- Renamed the orchestration module from pipeline.rs to extractor.rs and updated main.rs imports accordingly.
- Removed the now-empty nested module folders.

Tests:
- cargo fmt
- cargo test
<!-- SECTION:FINAL_SUMMARY:END -->
