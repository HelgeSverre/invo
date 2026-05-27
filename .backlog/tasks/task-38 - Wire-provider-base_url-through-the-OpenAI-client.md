---
id: TASK-38
title: Wire provider base_url through the OpenAI client
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:07'
labels:
  - code-review
  - llm
  - bug
  - security
dependencies: []
references:
  - src/llm.rs
priority: high
ordinal: 40000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/llm.rs:56 uses openai::Client::new(api_key)? which never consumes provider_config.base_url, while Ollama, Anthropic, and Mistral all use builders with .base_url(). Contradicts DECISION-3 and the agent-context active_keys advertisement. Corporate-proxy users configure base_url='https://proxy.corp.local/openai/v1' expecting traffic to go through the proxy and instead leak their API key to api.openai.com. Reviewers: maintainability, adversarial, agent-native, correctness.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 OpenAI client construction passes provider_config.base_url to a builder, mirroring build_anthropic_client
- [x] #2 When base_url is configured for OpenAI, requests target that endpoint (not api.openai.com)
- [x] #3 Regression test mirrors the existing ollama_uses_configured_base_url shape for OpenAI
<!-- AC:END -->
