---
id: TASK-63
title: Share the rig-core HTTP client across batch items
status: Done
assignee: []
created_date: '2026-05-27 00:11'
updated_date: '2026-05-27 04:00'
labels:
  - code-review
  - performance
  - llm
  - refactor
dependencies:
  - TASK-29
references:
  - src/llm.rs
priority: low
ordinal: 65000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/llm.rs:49-71 builds a new provider client (build_ollama_client, openai::Client::new, etc.) on every call to extract_invoice_data. For a sequential batch this discards reqwest's connection pool between files and pays TCP handshake overhead per request. Naturally positions for TASK-29 (parallel batching). Reviewer: performance.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Provider client is built once per process and reused across batch items via an extractor context struct
- [ ] #2 Connection-pool reuse is observable in logs/traces or asserted in a focused test
- [x] #3 Refactor lands in a shape compatible with TASK-29 (parallel batching)
<!-- AC:END -->



## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Introduced LlmContext that builds the rig provider client once and reuses it across batch items. main.rs builds the context once before the per-file loop; extract_invoice takes &LlmContext instead of Option<ProviderName>. Connection-pool reuse drops out for free because each provider's reqwest client now lives across calls. Removed dead extract_invoice_data shim. The resolved Provider is exposed via .provider() so OCR mode=same resolves against the cached metadata instead of re-walking config. Naturally positions for TASK-29 parallel batching: share &LlmContext across spawn_blocking / buffer_unordered tasks.
<!-- SECTION:NOTES:END -->
