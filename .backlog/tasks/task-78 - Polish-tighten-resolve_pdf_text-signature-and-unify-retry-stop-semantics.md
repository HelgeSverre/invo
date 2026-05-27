---
id: TASK-78
title: 'Polish: tighten resolve_pdf_text signature and unify retry-stop semantics'
status: Done
assignee: []
created_date: '2026-05-27 13:24'
updated_date: '2026-05-27 14:19'
labels:
  - refactor
  - refactor-followup
dependencies: []
references:
  - src/extractor.rs
  - src/llm.rs
priority: low
ordinal: 80000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Two Minor follow-ups flagged by the review of 01cc163 (F1 extraction):

**1. resolve_pdf_text signature** — currently takes `llm: &LlmContext` but only reads `llm.provider()` and `llm.http()`. A tighter signature `resolve_pdf_text(input, config, http, provider) -> Result<PdfText>` would make the dependency explicit and let the helper be unit-tested in isolation (today it requires a fully-built LlmContext just to construct the test).

**2. RetryDecision::PermanentStop variant** — `classify_retry` returns `Stop` for both retry exhaustion AND permanent errors, then `complete_invoice` re-calls `is_permanent_error` in the `Stop` arm to decide whether to emit the warn. Splitting into `RetryDecision::Stop | PermanentStop` removes the double call and surfaces the distinction in one place.

Both are <10 LOC each. Bundle into one polish commit when convenient.

Surface area:
- src/extractor.rs:20-56 (resolve_pdf_text)
- src/llm.rs:271-302 (RetryDecision, classify_retry) and 305-345 (complete_invoice retry loop)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 resolve_pdf_text takes &reqwest::Client and Provider arguments instead of &LlmContext
- [x] #2 RetryDecision gains a PermanentStop variant; complete_invoice's Stop arm no longer calls is_permanent_error
- [x] #3 Existing tests (114 unit + 16 integration + 3 classify_retry unit tests) all pass
- [x] #4 New unit test added for classify_retry returning PermanentStop on a permanent error
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented in commit pending. resolve_pdf_text now takes &reqwest::Client + &Provider directly; LlmContext is no longer needed by the helper. RetryDecision gained PermanentStop variant — classify_retry now returns it for permanent errors regardless of remaining attempts, and complete_invoice's Stop arm no longer re-calls is_permanent_error. New unit test classify_retry_permanent_wins_over_exhausted_budget locks in the permanent-takes-precedence semantics. DoD #4 (docs) and #5 (notes) not applicable — no behavior or user-facing output changed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied and checked
- [x] #2 Relevant tests are added or updated
- [x] #3 cargo fmt, cargo test, and cargo clippy --all-targets -- -D warnings pass
- [ ] #4 Documentation/help/config examples are updated when behavior or user-facing output changes
- [ ] #5 Backlog task notes and final summary describe what changed and how it was verified
- [x] #6 Self-review confirms no unrelated changes, secrets, or debug artifacts are included
<!-- DOD:END -->
