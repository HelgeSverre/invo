---
id: TASK-58
title: Bound retry_attempts to prevent unbounded request storms
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:10'
labels:
  - code-review
  - reliability
  - config
dependencies: []
references:
  - src/config.rs
priority: medium
ordinal: 60000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/config.rs:167 declares retry_attempts: u32 with no upper bound. A configuration typo of 4_294_967_295 — or a deliberately malicious config — yields 4.3 billion sequential requests with no backoff. Pairs with the retry-backoff task and amplifies its impact. Reviewer: adversarial.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 retry_attempts is capped at a sensible max (e.g. 10) via validation in parse_config_str or a custom deserializer (custom deserializer, MAX_RETRY_ATTEMPTS = 10)
- [x] #2 Out-of-range values produce a clear parse error citing the bound ("llm.extraction.retry_attempts must be <= 10, got <value>")
- [x] #3 Unit test pins the rejection (rejects_retry_attempts_above_bound + accepts_retry_attempts_at_bound)
<!-- AC:END -->
