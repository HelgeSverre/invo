---
id: TASK-35
title: Add OCR fallback for scanned PDFs via Ollama vision (rasterization)
status: Done
assignee: []
created_date: '2026-05-27 00:04'
updated_date: '2026-05-27 00:43'
labels:
  - feature
  - pdf
  - ocr
  - vision
  - ollama
dependencies:
  - TASK-28
modified_files:
  - Cargo.toml
  - src/ocr.rs
  - README.md
priority: low
ordinal: 37000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Follow-up to TASK-28. Extend the OCR fallback to support processing.ocr.mode = llm with name: ollama and mode: same when the extractor provider is Ollama. Unlike Mistral/Anthropic/OpenAI, Ollama vision models accept images only, so this task requires client-side PDF rasterization. This re-introduces a native dependency (PDFium via pdfium-render or equivalent) that TASK-28 deliberately avoided. Only land this when there is real user demand, and keep provider-specific OCR infrastructure outside extractor orchestration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `processing.ocr.mode = llm` with `name: ollama` rasterizes a scanned PDF and submits page images to a configured vision model.
- [x] #2 PDFium dependency story is clearly documented (README "Enabling Ollama OCR" lists per-platform install commands and links to github.com/bblanchon/pdfium-binaries).
- [x] #3 Default cross-platform build is unaffected — PDFium is gated behind the optional `ollama-ocr` Cargo feature. Default build pulls zero PDFium/image deps into the compiled binary.
- [x] #4 README OCR section calls out Ollama as the only backend needing native deps.
- [x] #5 Tests cover both feature configurations: with-feature unit tests assert `resolve_ocr` produces `ResolvedOcr::Ollama`; without-feature test asserts `ocr_via_ollama` returns the actionable rebuild error. Live test (`ollama_ocr_extracts_text_from_sample_pdf`) is gated on `--features ollama-ocr` + `#[ignore]`.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
`ocr_via_ollama` is feature-gated behind `ollama-ocr` (Cargo feature, off by default). With the feature: rasterizes PDF pages to PNGs via `pdfium-render` 0.8 inside a `tokio::task::spawn_blocking` (Pdfium is `!Send`), target width 1536px, base64-encoded, POSTed to `<base_url>/api/chat` with `model`, `stream: false`, and `messages[0].images: [base64png...]`. Response text taken from `message.content`. Defaults: `llama3.2-vision`, `http://localhost:11434`. Without the feature: `ocr_via_ollama` returns an actionable rebuild error pointing at `cargo install --features ollama-ocr`. PDFium installation documented in the README OCR section.
<!-- SECTION:NOTES:END -->
