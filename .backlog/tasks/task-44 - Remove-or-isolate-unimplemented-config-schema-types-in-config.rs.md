---
id: TASK-44
title: Remove or isolate unimplemented config schema types in config.rs
status: Done
assignee: []
created_date: '2026-05-27 00:08'
updated_date: '2026-05-27 04:31'
labels:
  - code-review
  - maintainability
  - config
dependencies: []
references:
  - src/config.rs
  - .backlog/decisions
modified_files:
  - src/config.rs
  - src/config_commands.rs
  - src/agent_context.rs
  - src/llm.rs
  - src/extractor.rs
  - README.md
priority: high
ordinal: 46000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/config.rs still contains fully typed future config surfaces that have no runtime consumers, especially actions, deduplication, logging, and provider model_path. OCR is now partially active and should no longer be treated as parsed-only. The remaining inactive schema increases validation and migration burden because #[serde(deny_unknown_fields)] makes accepted-but-unimplemented fields part of the public contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Remaining unimplemented config types/fields are either removed, feature-gated, or isolated from the active runtime Config.
- [x] #2 OCR config remains in the active schema; actions/deduplication/logging/model_path are either implemented or clearly treated as future-only.
- [x] #3 config doctor, README, and agent-context agree on which keys are active versus parsed-only.
<!-- AC:END -->
