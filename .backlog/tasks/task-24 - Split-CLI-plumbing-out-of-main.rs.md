---
id: TASK-24
title: Split CLI plumbing out of main.rs
status: To Do
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-25 15:31'
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
priority: medium
ordinal: 26000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
main.rs now contains CLI parsing, config commands, agent context, input expansion, output rendering, and tests. Split these concerns into flat src/*.rs modules where the separation is now justified.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 main.rs primarily wires parsing, logging, and top-level dispatch.
- [ ] #2 Input collection and stdin/file-list handling live outside main.rs.
- [ ] #3 Output rendering lives outside main.rs.
- [ ] #4 Config subcommand handlers live outside main.rs.
- [ ] #5 No behavior changes are introduced.
- [ ] #6 Existing tests pass after the split.
<!-- AC:END -->
