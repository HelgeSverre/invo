---
id: TASK-31
title: Implement post-processing actions
status: Done
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 01:10'
labels:
  - feature
  - workflow
dependencies: []
modified_files:
  - src/actions.rs
  - src/config.rs
  - src/main.rs
  - src/output.rs
  - src/config_commands.rs
  - src/agent_context.rs
  - README.md
priority: low
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Config already models copy, move, export, hook, and notify actions, but they are parsed-only. Implement or deliberately narrow this post-processing surface.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Configured copy and move actions run after successful extraction.
- [ ] #2 Configured export actions write the selected output format to the configured destination.
- [ ] #3 Hook actions execute with documented arguments/environment and honor on_error.
- [ ] #4 Notify actions are either implemented or removed from accepted config.
- [ ] #5 config doctor, README, and agent-context no longer report implemented actions as parsed-only.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented Copy/Move/Export/Hook actions in new src/actions.rs module, wired into main.rs after each successful extraction. Notify removed from Action enum (configs that included it now fail strict validation; the doctor surfaces a targeted migration hint). Placeholder vocabulary: {vendor} {date} {year} {month} {day} {invoice_number} {currency} {total} {hash} {original_name} {stem}. Hook actions get INVO_* env vars and honor on_error: continue (logs and proceeds) or abort (propagates the error). Path separators in expanded filenames are sanitized to _. Doctor + agent-context updated: actions[] removed from parsed_only, individual subkeys added to active_keys. README has new 'Post-processing actions' section with one YAML example per type. 9 unit tests cover placeholders, sanitization, copy/move/export, hook env passing, on_error semantics. cargo clippy --all-targets clean (default and ollama-ocr); 65 unit + 12 integration tests pass.
<!-- SECTION:NOTES:END -->
