---
id: TASK-56
title: >-
  Extract validate_output_flags helper instead of discarding output_format
  result
status: To Do
assignee: []
created_date: '2026-05-27 00:10'
updated_date: '2026-05-27 00:44'
labels:
  - code-review
  - maintainability
  - refactor
dependencies: []
references:
  - src/main.rs
  - src/output.rs
priority: medium
ordinal: 58000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs still calls output::output_format early only for its error side effect and then render_results calls output_format again. The validation concern now spans main.rs and src/output.rs after the TASK-24 split. Extract a dedicated helper so validation and format computation are not coupled through a discarded return value.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 validate_output_flags(&cli) -> Result<()> is extracted to do only the conflict check
- [ ] #2 main.rs invokes validate_output_flags(&cli)? early instead of the let _ = pattern
- [ ] #3 output_format keeps its single responsibility of computing the format for render_results
<!-- AC:END -->
