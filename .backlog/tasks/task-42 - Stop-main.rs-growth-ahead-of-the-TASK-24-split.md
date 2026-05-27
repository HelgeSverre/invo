---
id: TASK-42
title: Stop main.rs growth ahead of the TASK-24 split
status: Done
assignee: []
created_date: '2026-05-27 00:08'
updated_date: '2026-05-27 00:43'
labels:
  - code-review
  - maintainability
  - refactor
dependencies:
  - TASK-24
references:
  - src/main.rs
modified_files:
  - src/main.rs
  - src/input.rs
  - src/output.rs
  - src/config_commands.rs
  - src/agent_context.rs
priority: high
ordinal: 44000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs went from 185 to 902 lines in this branch (4.9x growth, 98 lines from the 1k regression threshold). It now contains CLI definitions, all five ConfigCommand handlers, the extraction loop, input collection (collect_inputs/expand_inputs/read_files_from/parse_file_list), config-doctor engine, output rendering (render_results/render_json_results/render_yaml_results/render_csv_results/csv_escape), and a 90-line test module. TASK-24 is already planned to split this; this task is to land partial relief on or before this branch so the deferred split is not made harder. Reviewer: maintainability.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 main.rs is back under 600 lines, or TASK-24 lands first as a precondition
- [x] #2 Extracted modules follow the TASK-24 plan: config commands, input collection, output rendering
- [x] #3 No new top-level subcommand handlers are added inline to main.rs in this branch
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Resolved by TASK-24. main.rs has been split into focused input, output, config command, and agent-context modules; main.rs is now primarily CLI definitions, logging, dispatch, and extraction orchestration.
<!-- SECTION:FINAL_SUMMARY:END -->
