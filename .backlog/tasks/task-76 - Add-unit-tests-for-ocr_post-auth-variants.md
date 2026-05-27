---
id: TASK-76
title: Add unit tests for ocr_post auth variants
status: Done
assignee: []
created_date: '2026-05-27 13:24'
updated_date: '2026-05-27 14:22'
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
- [x] #1 ocr_post is exercised by at least one test per Auth variant against a mock server
- [x] #2 Each test asserts the request headers reach the server correctly (e.g. Authorization: Bearer X for Auth::Bearer; x-api-key + anthropic-version for AnthropicKey; no auth header for None)
- [x] #3 At least one test covers the network-error path and at least one covers the error_for_status path with the wrapped context message present
- [x] #4 Tests pass with cargo test and do not depend on live network access
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added 6 wiremock-based unit tests for ocr_post in src/extractor.rs covering: Bearer auth header attachment, AnthropicKey + anthropic-version header attachment, Auth::None header absence (verified via received_requests inspection), and the three error-context wrappers (network / error_for_status / malformed JSON). Tests do not require network access — wiremock spins per-test MockServer on an ephemeral port. Added wiremock 0.6.5 as a dev-dependency. Test count: 118 + 16 -> 124 + 16. DoD #4 (docs) and #5 (notes) not applicable — no behavior or user-facing output changed.
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
