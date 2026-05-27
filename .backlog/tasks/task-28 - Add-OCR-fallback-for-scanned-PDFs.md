---
id: TASK-28
title: Add OCR fallback for scanned PDFs
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 01:51'
labels:
  - feature
  - pdf
  - ocr
dependencies: []
modified_files:
  - Cargo.toml
  - Cargo.lock
  - src/config.rs
  - src/extractor.rs
  - src/llm.rs
  - src/pdf.rs
  - src/config_commands.rs
  - src/agent_context.rs
  - config.yaml
  - README.md
priority: medium
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Recover text from image-only / scanned PDFs by routing them through a vision/document-model API when native PDF text extraction yields too little text. Cross-platform, no native deps (no tesseract, no poppler, no PDFium). v1 ships Mistral OCR as the only fully implemented backend; Anthropic/OpenAI/Ollama OCR backends are separate follow-up work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Image-only or scanned PDFs can produce extracted text through OCR when OCR is enabled.
- [ ] #2 The output extraction.ocr_used field is true when OCR is used.
- [ ] #3 Users get actionable errors when OCR is unavailable or disabled.
- [ ] #4 README documents OCR requirements and limitations.
- [ ] #5 Tests cover OCR-disabled and OCR-unavailable behavior without requiring system OCR in default test runs.
- [ ] #6 check 1,2,3,4,5
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Keep PDF native extraction byte-oriented in src/pdf.rs and detect short native text with MIN_MEANINGFUL_TEXT_LENGTH.
2. Model OCR configuration in src/config.rs with mode: none | mistral | same | llm, defaulting to none.
3. Resolve OCR configuration before the invoice LLM call: disabled mode returns an actionable scanned-PDF error; Mistral modes resolve model and API key environment variables.
4. Keep config-doctor and agent-context updates in src/config_commands.rs and src/agent_context.rs after the TASK-24 split.
5. Prefer moving provider-specific OCR HTTP code into a dedicated OCR module before extending beyond Mistral, so src/extractor.rs remains orchestration-focused.
6. Document supported modes, request limits, and costs in README/config.yaml.
7. Cover resolve_ocr paths with unit tests and keep live OCR checks behind #[ignore] and API-key gating.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Mistral OCR fully implemented in src/extractor.rs::ocr_via_mistral, with resolve_ocr() routing OcrConfig::None/Mistral/Same/Llm. Anthropic/OpenAI/Ollama landed via TASK-33/34/35. extraction.ocr_used flips to true when OCR runs. mode=none + scanned PDF returns 'PDF appears scanned (only N chars). Configure processing.ocr.mode to enable OCR (mistral | anthropic | openai | ollama).' README has dedicated 'OCR fallback' section with backend matrix + four mode examples + Ollama install path. Default cargo test stays hermetic — live tests gated #[ignore] on api-key env vars. Test count: 8 resolve_ocr unit tests, 4 live #[ignore] tests, no system OCR required for default runs.
<!-- SECTION:NOTES:END -->
