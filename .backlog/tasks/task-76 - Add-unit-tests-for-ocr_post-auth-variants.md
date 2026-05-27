---
id: TASK-76
title: Add unit tests for ocr_post auth variants
status: To Do
assignee: []
created_date: '2026-05-27 13:24'
labels:
  - test
  - refactor-followup
dependencies: []
references:
  - src/extractor.rs
priority: low
ordinal: 78000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The shared OCR POST helper `ocr_post` introduced in 5b830dc is only exercised via the four `#[ignore]`d live OCR tests (which require real API keys and network access). The three `Auth` variants (`Bearer`, `AnthropicKey`, `None`) and the error-context wiring (network / error_for_status / json) have no unit coverage.

If `ocr_post` ever grows (redirect policy, retry, streaming, additional auth schemes), regressions will only show up in live tests. Add mockito-based or httpmock-based unit tests so the auth dispatch + error wrapping is locked in.

Surface area: `src/extractor.rs` `Auth`, `ocr_post`, the helper at lines ~107-138.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ocr_post is exercised by at least one test per Auth variant against a mock server
- [ ] #2 Each test asserts the request headers reach the server correctly (e.g. Authorization: Bearer X for Auth::Bearer; x-api-key + anthropic-version for AnthropicKey; no auth header for None)
- [ ] #3 At least one test covers the network-error path and at least one covers the error_for_status path with the wrapped context message present
- [ ] #4 Tests pass with cargo test and do not depend on live network access
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied and checked
- [ ] #2 Relevant tests are added or updated
- [ ] #3 cargo fmt, cargo test, and cargo clippy --all-targets -- -D warnings pass
- [ ] #4 Documentation/help/config examples are updated when behavior or user-facing output changes
- [ ] #5 Backlog task notes and final summary describe what changed and how it was verified
- [ ] #6 Self-review confirms no unrelated changes, secrets, or debug artifacts are included
<!-- DOD:END -->
