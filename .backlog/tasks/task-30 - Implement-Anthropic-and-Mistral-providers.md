---
id: TASK-30
title: Implement Anthropic and Mistral providers
status: Done
assignee:
  - '@agent'
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 17:31'
labels:
  - feature
  - providers
dependencies: []
modified_files:
  - src/llm.rs
  - src/config.rs
  - README.md
priority: medium
ordinal: 32000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The CLI accepts anthropic and mistral provider names but currently returns a not-implemented error. Implement them through Rig or narrow the accepted provider set.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Anthropic provider entries can run extraction when configured with model and api_key_env.
- [x] #2 Mistral provider entries can run extraction when configured with model and api_key_env.
- [x] #3 Missing credential errors are provider-specific and actionable.
- [x] #4 README provider table reflects the implemented status.
- [x] #5 Tests cover provider selection and missing credential behavior without live API calls.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Implement Anthropic provider using rig_core::providers::anthropic
2. Implement Mistral provider using rig_core::providers::mistral
3. Add proper error handling for missing API keys
4. Update README provider table
5. Add tests for both providers
6. Ensure base_url support works for both new providers
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented Anthropic and Mistral providers using rig_core. Both providers now work with proper API key handling and error messages. Added comprehensive tests for provider selection, API key requirements, and base_url support. Updated README with provider status and configuration documentation. All tests pass and the implementation follows the same pattern as the existing OpenAI provider.
<!-- SECTION:FINAL_SUMMARY:END -->
