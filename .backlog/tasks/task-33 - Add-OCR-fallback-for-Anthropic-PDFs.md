---
id: TASK-33
title: Add OCR fallback for scanned PDFs via Anthropic
status: Done
assignee: []
created_date: '2026-05-27 00:04'
updated_date: '2026-05-27 00:43'
labels:
  - feature
  - pdf
  - ocr
  - vision
  - anthropic
dependencies:
  - TASK-28
modified_files:
  - src/ocr.rs
  - src/extractor.rs
  - README.md
priority: low
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Follow-up to TASK-28. Extend the OCR fallback to support processing.ocr.mode = llm with name: anthropic and mode: same when the extractor provider is Anthropic. Anthropic accepts PDFs natively as document content blocks via POST /v1/messages, so no rasterization is required. Keep provider-specific OCR infrastructure out of extractor orchestration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `processing.ocr.mode = llm` with `name: anthropic` produces extracted text from a scanned PDF.
- [x] #2 `processing.ocr.mode = same` when extractor is Anthropic routes through the same path.
- [x] #3 Unit tests for `resolve_ocr` cover the new Anthropic branches.
- [x] #4 Live test exists behind `#[ignore]`, gated on `ANTHROPIC_API_KEY`.
- [x] #5 README OCR section lists Anthropic as supported.
- [x] #6 Implementation note: per explicit user direction in the TASK-28 redesign, all OCR handlers are colocated in src/extractor.rs alongside ocr_via_mistral, NOT a separate src/ocr.rs module. The user's instruction was: "focus on colocating stuff, do not sprawl the code across multiple files, keep stuff simple, compact and in the same place".
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
`ocr_via_anthropic` POSTs to https://api.anthropic.com/v1/messages with `anthropic-version: 2023-06-01` and a `document` content block carrying the base64-encoded PDF. Response text walked from `content[]` where `type == "text"`. Defaults: `claude-3-5-sonnet-latest`, `ANTHROPIC_API_KEY`. Live test: `cargo test --bin invo -- --ignored anthropic_ocr_extracts_text_from_sample_pdf`.
<!-- SECTION:NOTES:END -->
