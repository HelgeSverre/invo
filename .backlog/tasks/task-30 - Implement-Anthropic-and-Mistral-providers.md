---
id: TASK-30
title: Implement Anthropic and Mistral providers
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
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
- [ ] #1 Anthropic provider entries can run extraction when configured with model and api_key_env.
- [ ] #2 Mistral provider entries can run extraction when configured with model and api_key_env.
- [ ] #3 Missing credential errors are provider-specific and actionable.
- [ ] #4 README provider table reflects the implemented status.
- [ ] #5 Tests cover provider selection and missing credential behavior without live API calls.
<!-- AC:END -->
