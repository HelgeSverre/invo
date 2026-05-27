---
id: TASK-34
title: Add OCR fallback for scanned PDFs via OpenAI
status: Done
assignee: []
created_date: '2026-05-27 00:04'
updated_date: '2026-05-27 00:43'
labels:
  - feature
  - pdf
  - ocr
  - vision
  - openai
dependencies:
  - TASK-28
modified_files:
  - src/ocr.rs
  - src/extractor.rs
  - README.md
priority: low
ordinal: 36000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Follow-up to TASK-28. Extend the OCR fallback to support processing.ocr.mode = llm with name: openai and mode: same when the extractor provider is OpenAI. Use the Responses API with an input_file content block carrying the base64 PDF inline — no Files-API roundtrip, no rasterization. Keep provider-specific OCR infrastructure out of extractor orchestration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `processing.ocr.mode = llm` with `name: openai` produces extracted text from a scanned PDF via Responses API.
- [x] #2 `processing.ocr.mode = same` when extractor is OpenAI routes through the same path.
- [x] #3 Unit tests for `resolve_ocr` cover the new OpenAI branches.
- [x] #4 Live test exists behind `#[ignore]`, gated on `OPENAI_API_KEY`.
- [x] #5 README OCR section lists OpenAI as supported.
- [x] #6 Implementation note: per explicit user direction in the TASK-28 redesign, all OCR handlers are colocated in src/extractor.rs alongside ocr_via_mistral, NOT a separate src/ocr.rs module.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
`ocr_via_openai` POSTs to https://api.openai.com/v1/responses with `Authorization: Bearer`, sending the PDF inline via an `input_file` content part (no Files-API roundtrip). Response text taken from top-level `output_text` when present, otherwise walked from `output[].content[]` where `type ∈ {output_text, text}`. Note: this is the Responses API, distinct from the Chat Completions endpoint that Rig uses for extraction — divergence is intentional and called out in the code comment. Defaults: `gpt-4o`, `OPENAI_API_KEY`. Live test: `cargo test --bin invo -- --ignored openai_ocr_extracts_text_from_sample_pdf`.
<!-- SECTION:NOTES:END -->
