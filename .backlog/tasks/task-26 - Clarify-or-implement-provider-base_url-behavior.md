---
id: TASK-26
title: Clarify or implement provider base_url behavior
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
labels:
  - providers
  - config
dependencies: []
modified_files:
  - src/llm.rs
  - src/main.rs
  - src/config.rs
  - README.md
priority: medium
ordinal: 28000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The config treats llm.providers[].base_url as active, but the implementation only uses it for Ollama. Decide whether to support OpenAI-compatible base URLs or document/diagnose that base_url is Ollama-only today.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Either OpenAI base_url is supported when configured, or docs and diagnostics clearly state base_url is Ollama-only.
- [ ] #2 README, config doctor, config schema descriptions, and agent-context agree on the chosen behavior.
- [ ] #3 Tests cover the chosen base_url behavior.
<!-- AC:END -->
