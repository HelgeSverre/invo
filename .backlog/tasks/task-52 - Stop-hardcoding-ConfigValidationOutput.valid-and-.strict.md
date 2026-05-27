---
id: TASK-52
title: Stop hardcoding ConfigValidationOutput.valid and .strict
status: Done
assignee: []
created_date: '2026-05-27 00:09'
labels:
  - code-review
  - bug
  - cli
  - config
dependencies: []
references:
  - src/main.rs
priority: medium
ordinal: 54000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs:267-297 constructs ConfigValidationOutput { valid: true, strict: true } regardless of the CLI --strict flag or actual validation outcome. The strict parameter on the function is used only to emit a tracing::debug log, never to populate the struct. JSON consumers of 'config validate --json' see {"strict": true} even when --strict was not passed. Reviewer: maintainability.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ConfigValidationOutput.strict reflects the strict parameter passed to handle_config_validate
- [x] #2 ConfigValidationOutput.valid reflects whether parsing actually succeeded (handler `?`'s on parse failure, so reaching emission implies valid; documented with comment)
- [x] #3 Either --strict is wired to meaningful behavior, or the CLI flag is removed (flag now drives output.strict and the human-readable "Strict validation:" line)
- [x] #4 Unit test pins behavior for both --strict and bare invocation
<!-- AC:END -->
