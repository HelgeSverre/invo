---
id: TASK-54
title: Mark common CLI flags global so they work after subcommand names
status: To Do
assignee: []
created_date: '2026-05-27 00:09'
labels:
  - code-review
  - cli
  - agent
dependencies:
  - TASK-10
references:
  - src/main.rs
priority: medium
ordinal: 56000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs:39-65 marks only --json and --config as global = true. Invocations like 'invo config validate -v' or 'invo config doctor --format json' fail with a Clap 'unexpected argument' error. TASK-10 fixed only --config. Agents and humans who tail flags onto subcommand names hit unexpected failures. Reviewer: agent-native.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 --verbose is marked global = true and works after every subcommand name
- [ ] #2 agent-context output and CLI help document the required placement for any flag that cannot be global (--format, --json-array, --pretty, --provider) and includes a working example
- [ ] #3 Regression test asserts 'invo config validate -v' parses without error
<!-- AC:END -->
