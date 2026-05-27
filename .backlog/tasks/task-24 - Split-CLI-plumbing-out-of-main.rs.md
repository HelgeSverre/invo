---
id: TASK-24
title: Split CLI plumbing out of main.rs
status: Done
assignee:
  - '@pi'
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 00:45'
labels:
  - refactor
  - cli
dependencies: []
modified_files:
  - src/main.rs
  - src/input.rs
  - src/output.rs
  - src/config_commands.rs
  - src/agent_context.rs
  - src/pdf.rs
  - src/extractor.rs
priority: medium
ordinal: 26000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
main.rs now contains CLI parsing, config commands, agent context, input expansion, output rendering, and tests. Split these concerns into flat src/*.rs modules where the separation is now justified.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 main.rs primarily wires parsing, logging, and top-level dispatch.
- [x] #2 Input collection and stdin/file-list handling live outside main.rs.
- [x] #3 Output rendering lives outside main.rs.
- [x] #4 Config subcommand handlers live outside main.rs.
- [x] #5 No behavior changes are introduced.
- [x] #6 Existing tests pass after the split.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Refactored CLI plumbing into focused modules and narrowed main.rs to parsing, logging, top-level dispatch, and extraction orchestration. Input collection lives in src/input.rs, output rendering in src/output.rs, config handlers in src/config_commands.rs, and agent-context generation in src/agent_context.rs. Also simplified pdf.rs to byte-level extraction and normalized extractor inputs around bytes. Validation: cargo test and cargo clippy --all-targets -- -D warnings pass.
<!-- SECTION:FINAL_SUMMARY:END -->
