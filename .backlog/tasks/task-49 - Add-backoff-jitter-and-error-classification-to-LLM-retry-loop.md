---
id: TASK-49
title: Add backoff/jitter and error classification to LLM retry loop
status: To Do
assignee: []
created_date: '2026-05-27 00:09'
labels:
  - code-review
  - reliability
  - performance
  - llm
dependencies: []
references:
  - src/llm.rs
priority: medium
ordinal: 51000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/llm.rs:131-148 retries immediately with no delay; with the default retry_attempts=2 three back-to-back requests fire against an overloaded provider. It also retries on permanent errors (401 auth, 400 schema-unsupported) treating them like transient network errors, racking up token cost and triggering 429 storms. Reviewers: performance, reliability, adversarial.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Retries use exponential backoff with jitter (e.g. 500ms * 2^attempt, +/- 0-200ms)
- [ ] #2 Permanent error categories (401/403, 400 with schema-unsupported, 404 model) are not retried
- [ ] #3 Backoff and per-attempt delay are visible in tracing output
- [ ] #4 Regression test asserts no retry happens on a 401-shaped error
<!-- AC:END -->
