---
id: TASK-61
title: Make assistant_text resilient to future ToolCall-shaped LLM responses
status: To Do
assignee: []
created_date: '2026-05-27 00:10'
labels:
  - code-review
  - reliability
  - llm
dependencies: []
references:
  - src/llm.rs
priority: medium
ordinal: 63000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/llm.rs:176-192 assistant_text filter_maps AssistantContent::Text only; ToolCall variants are silently discarded. Anthropic native structured output is typically delivered as a tool_use block. If rig-core's Anthropic adapter ever surfaces structured output as ToolCall (entirely a function of rig-core internals which we do not control), the extracted JSON is dropped, retries fire pointlessly, and the user gets 'No text content from LLM response' — even though Claude successfully extracted the invoice. Reviewers: adversarial, correctness.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 When no Text content is present in AssistantContent, assistant_text falls back to scanning ToolCall arguments (e.g. the first ToolCall whose name matches the schema function)
- [ ] #2 Alternatively, an explicit assertion is added with a clear error message tying it to the rig-core version
- [ ] #3 Behavior on today's Text-only responses is unchanged
<!-- AC:END -->
