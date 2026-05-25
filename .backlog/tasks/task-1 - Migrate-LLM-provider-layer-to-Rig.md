---
id: TASK-1
title: Migrate LLM provider layer to Rig
status: Done
assignee:
  - '@codex'
created_date: '2026-05-25 12:56'
updated_date: '2026-05-25 13:21'
labels:
  - llm
  - refactor
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - Cargo.toml
  - Cargo.lock
  - src/llm/mod.rs
  - src/llm/ollama.rs
  - src/llm/openai.rs
  - src/llm/provider.rs
  - src/main.rs
  - src/pipeline.rs
  - src/extraction/mod.rs
  - src/extraction/prompt.rs
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the hand-rolled OpenAI/Ollama clients and local LlmProvider abstraction with Rig-backed extraction while preserving current extraction behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ollama-rs, direct OpenAI reqwest chat code, and custom provider request/response structs are removed unless still required outside LLM calls.
- [x] #2 Ollama and OpenAI extraction work through Rig with existing config values.
- [x] #3 ExtractionResult still includes provider and model metadata.
- [x] #4 cargo build passes.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add focused unit tests around provider selection and missing credential behavior so the migration has a regression harness.
2. Add Rig as the LLM dependency and remove direct provider crates that are no longer used.
3. Replace the current LlmProvider trait implementations with a Rig-backed provider wrapper that preserves create_provider and pipeline call sites initially.
4. Keep Ollama and OpenAI config semantics stable for TASK-1: existing provider entries, model, base_url, api_key_env, temperature, max_tokens, and retry_attempts continue to drive extraction.
5. Run cargo test and cargo build, then mark acceptance criteria that are satisfied.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Migrated extraction to Rig completion models for Ollama and OpenAI. The old provider trait plus hand-written Ollama/OpenAI HTTP clients were removed. Rig now receives the configured model, Ollama base_url fallback, OpenAI api_key_env, temperature, max_tokens, and retry_attempts. Added unit coverage for provider selection, missing API key config, Rig request settings/schema, text extraction, and retry behavior.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
TASK-1 replaces the custom LLM provider layer with Rig-backed structured completion. Ollama and OpenAI now run through rig-core, ExtractionResult still records provider/model metadata, and direct ollama-rs/reqwest/async-trait provider code has been removed. Verified with cargo test, cargo build, and CLI help smoke.
<!-- SECTION:FINAL_SUMMARY:END -->
