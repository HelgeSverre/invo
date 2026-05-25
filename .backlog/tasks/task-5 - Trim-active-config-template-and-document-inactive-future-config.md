---
id: TASK-5
title: Trim active config template and document inactive future config
status: Done
assignee:
  - '@codex'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 13:32'
labels:
  - config
  - ux
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - config.yaml
  - README.md
priority: high
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Keep the generated config.yaml honest by including only active runtime keys and moving future settings out of the default template.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Parsed-but-unused sections are removed from generated config or moved to clearly marked future/example docs.
- [x] #2 OCR, retry, action, parallelism, dedupe, and logging contradictions are removed from the default template.
- [x] #3 Config docs include a status table for active versus future keys.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Trim config.yaml to settings that currently affect runtime behavior.
2. Keep retry_attempts only if verified active through the Rig migration; remove inactive OCR/action/parallelism/dedupe/logging/output metadata from the generated template.
3. Update README configuration docs with an active/future status table.
4. Validate config.yaml and run cargo test/build.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Trimmed generated config.yaml to active runtime settings: identity fields used in prompts, provider/model/base_url/api_key_env, extraction temperature/max_tokens/retry_attempts, vendor name/patterns/category, and output.format. Removed inactive generated sections for ignored vendors, actions, processing/OCR/dedupe, logging, output metadata, address/vat, and vendor currency. README now documents active versus future parsed keys in a status table.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
TASK-5 keeps invo init honest by generating a smaller active-only config.yaml and documenting future parsed-only keys in README. retry_attempts remains in the template because TASK-1 made it active. Verified with cargo test, cargo build, config validation, and a temporary-HOME init smoke comparing generated config to config.yaml.
<!-- SECTION:FINAL_SUMMARY:END -->
