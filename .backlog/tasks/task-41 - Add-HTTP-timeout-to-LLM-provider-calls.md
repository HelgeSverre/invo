---
id: TASK-41
title: Add HTTP timeout to LLM provider calls
status: Done
assignee:
  - '@agent'
created_date: '2026-05-27 00:07'
updated_date: '2026-05-27 03:04'
labels:
  - code-review
  - reliability
  - llm
dependencies: []
references:
  - src/llm.rs
priority: high
ordinal: 43000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
rig-core 0.37 ClientBuilder defaults to reqwest::Client::default() which sets no timeout. A hung Ollama or unresponsive cloud endpoint will block the tokio task indefinitely; nothing in src/llm.rs wraps the .send().await in tokio::time::timeout. Reviewer: reliability.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All four provider client paths use a reqwest::Client built with a configurable timeout
- [x] #2 Timeout is exposed in config (e.g. llm.extraction.timeout_secs) with sensible defaults: ~60s for hosted APIs, ~120s for local Ollama
- [x] #3 A regression test using a hanging MockCompletionModel confirms the timeout fires
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add timeout_secs: Option<u64> to ExtractionSettings in src/config.rs (None = use provider-based default).
2. Add resolve_timeout(provider, settings) helper: returns Duration. Default 120s for Ollama, 60s for hosted APIs when None.
3. Update build_*_client fns to accept Duration; construct reqwest::Client with .timeout(duration) and pass via .http_client(client).
4. Store resolved Duration on LlmContext; thread it to complete_invoice / complete_invoice_once.
5. Wrap model.send().await in tokio::time::timeout(duration, ..) and surface 'LLM request timed out after Xs' on elapsed.
6. Add hanging-model regression test: tiny CompletionModel impl that never returns; assert complete_invoice_once errors within 2x timeout with 'timed out'.
7. Update config.yaml comment to document the new field.
8. cargo test + cargo clippy --all-targets -- -D warnings.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Wrap LLM provider calls in a configurable per-request timeout so a hung Ollama daemon or unresponsive cloud endpoint can no longer wedge the extraction task indefinitely.

Changes:
- Added optional llm.extraction.timeout_secs to ExtractionSettings. When unset, resolve_timeout() picks 120s for Ollama and 60s for hosted providers (openai / anthropic / mistral) at LlmContext::build time.
- Each provider client builder (build_ollama_client / build_openai_client / build_anthropic_client / build_mistral_client) now constructs a reqwest::Client with .timeout(...) and passes it to the rig ClientBuilder via .http_client(...), bounding the underlying TCP/TLS transport.
- complete_invoice_once wraps the .send() future in tokio::time::timeout(timeout, ...) so even an in-process model that never resolves can no longer block the task. Timeouts surface as 'LLM request timed out after Ns' and flow through the existing retry loop.
- Bumped reqwest to 0.13 to align with rig-core 0.37's HttpClientExt impl (the rustls feature replaces the now-removed rustls-tls feature name). Mistral OCR direct calls in extractor.rs are unaffected — Client::new / post / bearer_auth / json / send / error_for_status are stable across the bump.
- Documented the new field in config.yaml.

Tests:
- New: complete_invoice_errors_when_send_hangs_past_timeout — a HangingModel that never returns from completion() must time out under a 50ms budget in under 2s with a 'timed out' error.
- New: timeout_defaults_differ_for_ollama_and_hosted_providers, timeout_config_override_applies_to_every_provider, llm_context_exposes_resolved_timeout.
- cargo test → 101 lib tests + 15 cli tests pass.
- cargo clippy --all-targets -- -D warnings → clean.

Risks / follow-ups:
- The Mistral OCR direct reqwest calls in src/extractor.rs are still unbounded; tracked separately if reliability work continues.
<!-- SECTION:FINAL_SUMMARY:END -->
