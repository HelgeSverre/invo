---
id: TASK-16
title: Update README and docs after behavior changes
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 15:00'
labels:
  - docs
dependencies:
  - TASK-1
  - TASK-2
  - TASK-3
  - TASK-4
  - TASK-5
  - TASK-6
  - TASK-7
  - TASK-8
  - TASK-9
  - TASK-10
  - TASK-11
  - TASK-12
  - TASK-13
  - TASK-14
  - TASK-15
references:
  - REVIEW-CLI.md
modified_files:
  - README.md
priority: medium
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Align public docs with the fixed CLI and config behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 README reflects Rig-backed providers, config.yaml, output contracts, setup/config commands, and identity guidance.
- [x] #2 Inactive or future config values are not presented as current behavior.
- [x] #3 Examples are verified against the CLI.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review README against the final CLI behavior from tasks 1-15. 2. Update command/flag examples for stdin, --files-from, config subcommands, provider override, output contracts, and active vs future config keys. 3. Verify documented examples against the built CLI and record the checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Updated README for final task 1-15 behavior, including Rig-backed providers, config.yaml, stdout/stderr contracts, stdin, --files-from, config subcommands, and identity guidance.

Verified documented CLI examples that do not require an LLM by running help, config JSON, agent-context JSON, files-from, and stdin error smoke checks.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Aligned README with the finalized CLI and config behavior.

Changes:
- Documented Rig-backed providers, config.yaml setup, config subcommands, provider override semantics, output contracts, and buyer identity context.
- Added stdin and --files-from examples for scripts/agents.
- Kept inactive/future config values out of current behavior claims and documented them as parsed-only with config doctor guidance.
- Added stdout/stderr contract and agent/script recommendations.

Tests:
- cargo build
- target/debug/invo --help
- target/debug/invo config --help
- target/debug/invo config validate -c config.yaml --json
- target/debug/invo config show --effective --json
- target/debug/invo agent-context --json
- target/debug/invo --files-from /tmp/invo-files.txt -v
- printf 'not a pdf' | target/debug/invo -
<!-- SECTION:FINAL_SUMMARY:END -->
