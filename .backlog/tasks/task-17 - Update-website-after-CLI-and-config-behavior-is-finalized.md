---
id: TASK-17
title: Update website after CLI and config behavior is finalized
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 15:02'
labels:
  - website
  - docs
dependencies:
  - TASK-16
references:
  - REVIEW-CLI.md
modified_files:
  - website/index.html
  - website/style.css
priority: medium
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Refresh the marketing site once CLI output, provider behavior, and config naming are settled.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Website no longer claims unsupported fallback chains, parallel processing, actions, dedupe, or Anthropic support unless implemented.
- [x] #2 Quickstart uses config.yaml and the new setup flow.
- [x] #3 Provider and config examples match README.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit website copy against README/current CLI behavior and remove unsupported claims. 2. Update quickstart, provider, batch, and config examples to match config.yaml, --json/--json-array, --files-from, provider override, and active config keys. 3. Keep the static HTML/CSS accessible and responsive, then verify the page references no unsupported fallback chains, parallel actions, dedupe, or Anthropic support.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audited website copy against current CLI behavior and removed unsupported fallback chain, parallel processing, actions, dedupe, and Anthropic-provider claims.

Updated quickstart/provider/config examples for config.yaml, --json/--json-array, --files-from, stdin, and OpenAI provider override semantics.

Applied small responsive CSS improvements for long command blocks and wrapping nav links.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Refreshed the marketing site to match the finalized CLI and config behavior.

Changes:
- Removed unsupported fallback-chain, parallel processing, actions, dedupe, and Anthropic-provider claims.
- Updated quickstart snippets for compact JSON, NDJSON/JSON array batch output, stdin input, --files-from, config validation, and OpenAI override semantics.
- Updated config sample to active config.yaml fields: identity, Rig-backed provider entries, extraction settings, vendors, and output format.
- Improved responsive behavior for command blocks and navigation wrapping.

Tests:
- HTML parse smoke check with Python html.parser
- cargo test
- cargo build
- git diff --check
- grep audit for unsupported website claims
<!-- SECTION:FINAL_SUMMARY:END -->
