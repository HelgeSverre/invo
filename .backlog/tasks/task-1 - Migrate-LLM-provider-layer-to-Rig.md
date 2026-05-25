---
id: TASK-1
title: Migrate LLM provider layer to Rig
status: To Do
assignee: []
created_date: '2026-05-25 12:56'
labels:
  - llm
  - refactor
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - Cargo.toml
  - src/llm/mod.rs
  - src/pipeline.rs
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the hand-rolled OpenAI/Ollama clients and local LlmProvider abstraction with Rig-backed extraction while preserving current extraction behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ollama-rs, direct OpenAI reqwest chat code, and custom provider request/response structs are removed unless still required outside LLM calls.
- [ ] #2 Ollama and OpenAI extraction work through Rig with existing config values.
- [ ] #3 ExtractionResult still includes provider and model metadata.
- [ ] #4 cargo build passes.
<!-- AC:END -->
