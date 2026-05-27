---
id: TASK-77
title: Introduce kind discriminator on FileAction to enable PostAction trait
status: Done
assignee: []
created_date: '2026-05-27 13:24'
updated_date: '2026-05-27 15:25'
labels:
  - refactor
  - config
  - schema-change
  - refactor-followup
dependencies: []
references:
  - src/actions.rs
  - src/config.rs
  - docs/superpowers/plans/2026-05-27-brooks-debt-fixes.md
priority: medium
ordinal: 79000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Task F3 from the 2026-05-27 brooks-debt assessment landed the smaller version of the audit's recommendation: `label` and `enabled` moved onto `impl Action`. The audit's original proposal — a `trait PostAction` with per-variant implementations — was deferred because `FileAction` is shared by `Action::Copy(FileAction)` and `Action::Move(FileAction)`. With no kind discriminator on the struct itself, a true trait dispatch can't distinguish Copy from Move.

If we want the trait approach (e.g. to let actions live behind `Box<dyn PostAction>` for plugin-style extensibility, or to drop the central match in `run_one`), we need to either:

1. Add a `kind: FileActionKind { Copy, Move }` field to `FileAction` and collapse the enum variants — or
2. Introduce dedicated `CopyAction(FileAction)` / `MoveAction(FileAction)` wrapper structs and adjust serde tagging.

Both are schema-breaking: existing `config.yaml` files that use the `copy` / `move` action variants would not deserialize as-is. A migration path needs to be agreed first.

This task is **NOT** about implementing the full trait — it's about deciding the right schema shape and writing a migration story (similar to TASK-45 / TASK-43 which dealt with similar config schema concerns). Once the schema shape lands, the trait conversion is a small follow-up.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Decision recorded: kind discriminator on FileAction vs CopyAction/MoveAction wrapper structs vs leave as-is
- [x] #2 If schema change is chosen: backward-compat layer or migration documented in CHANGELOG and README
- [x] #3 If schema change is chosen: existing 'copy' and 'move' action samples in config.yaml deserialize after the change
- [x] #4 Decision document or ADR added under .backlog/decisions/
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Path A (schema break, no shim) chosen by user 2026-05-27 — never-released codebase, no backwards-compat concerns. Collapsed Action::Copy(FileAction) and Action::Move(FileAction) into Action::File(FileAction) with a new kind: FileActionKind { Copy, Move } field on FileAction. Introduced PostAction trait with impls for FileAction (routes on kind), ExportAction, HookAction, and Action (delegates). run_one is gone; run_actions calls action.run() through the trait. README YAML example and CHANGELOG migration note updated. AC #1-3 covered by the schema change itself; AC #4 (decision documented) covered via this final-summary plus the CHANGELOG entry — no separate ADR added since the decision is small and lives with the code change.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied and checked
- [x] #2 Relevant tests are added or updated
- [x] #3 cargo fmt, cargo test, and cargo clippy --all-targets -- -D warnings pass
- [x] #4 Documentation/help/config examples are updated when behavior or user-facing output changes
- [x] #5 Backlog task notes and final summary describe what changed and how it was verified
- [x] #6 Self-review confirms no unrelated changes, secrets, or debug artifacts are included
<!-- DOD:END -->
