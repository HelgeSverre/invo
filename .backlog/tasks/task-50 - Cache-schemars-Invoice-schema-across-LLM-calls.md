---
id: TASK-50
title: Cache schemars Invoice schema across LLM calls
status: To Do
assignee: []
created_date: '2026-05-27 00:09'
labels:
  - code-review
  - performance
  - llm
dependencies: []
references:
  - src/llm.rs
priority: medium
ordinal: 52000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/llm.rs:165 calls schemars::schema_for!(Invoice) inside complete_invoice_once, which runs on every LLM call and every retry attempt. The Invoice schema is static. A batch of 10 files with retry_attempts=2 generates 30 schema walks where 1 would suffice. Reviewer: performance.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 schemars::schema_for!(Invoice) is invoked at most once per process via std::sync::OnceLock (or LazyLock) at module scope
- [ ] #2 complete_invoice_once takes a &'static reference to the cached schema instead of regenerating inline
- [ ] #3 Behavior is unchanged: the schema sent to the LLM is byte-identical to today
<!-- AC:END -->
