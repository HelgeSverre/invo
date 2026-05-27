---
id: TASK-41
title: Add HTTP timeout to LLM provider calls
status: To Do
assignee: []
created_date: '2026-05-27 00:07'
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
- [ ] #1 All four provider client paths use a reqwest::Client built with a configurable timeout
- [ ] #2 Timeout is exposed in config (e.g. llm.extraction.timeout_secs) with sensible defaults: ~60s for hosted APIs, ~120s for local Ollama
- [ ] #3 A regression test using a hanging MockCompletionModel confirms the timeout fires
<!-- AC:END -->
