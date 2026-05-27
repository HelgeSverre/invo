---
id: TASK-81
title: Detect parallel-action destination collisions
status: To Do
assignee: []
created_date: '2026-05-27 16:04'
labels:
  - feature
  - actions
  - task-29-followup
  - parallelism
dependencies: []
references:
  - src/actions.rs
priority: medium
ordinal: 83000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When processing.parallelism > 1, multiple extraction workers run actions::run_actions concurrently. If two workers' filename templates expand to the same filesystem destination (e.g., template '{vendor}.pdf' with two invoices from the same vendor in the same batch), they race on writes.

Today the second write either overwrites the first (Copy/Move/Export with same path) or both succeed if writes interleave but neither sees the other's data. Hook actions are not affected (they don't write to a deterministic file path) unless the user wrote the destination into their hook script.

Decision needed (record as ADR):
- (a) Skip duplicates: first writer wins, second worker logs WARN and skips its file action
- (b) Abort batch: any detected collision propagates as an error and the batch exits with the partial-failure exit code
- (c) Warn-only: log loudly but allow the overwrite; surface in stderr summary

Implementation surface:
- Action::run path in src/actions.rs; or wrap run_actions with a coordinator
- Need a shared destination registry (Arc<Mutex<HashSet<PathBuf>>>) populated before write; check before write
- For Move actions the source-removal step interacts: if the second copy was skipped, the source PDF for the second one should NOT be deleted

Acceptance criteria below assume option (a) skip-with-WARN; revise if the ADR picks a different path.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ADR documenting the chosen behavior added under .backlog/decisions/
- [ ] #2 When two parallel workers resolve to the same Copy/Move/Export destination, behavior follows the ADR
- [ ] #3 When a Move worker is skipped due to collision, the source PDF is NOT removed
- [ ] #4 Test covers the collision path with parallelism: 2 and a template that produces identical destinations
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied and checked
- [ ] #2 Relevant tests are added or updated
- [ ] #3 cargo fmt, cargo test, and cargo clippy --all-targets -- -D warnings pass
- [ ] #4 Documentation/help/config examples are updated when behavior or user-facing output changes
- [ ] #5 Backlog task notes and final summary describe what changed and how it was verified
- [ ] #6 Self-review confirms no unrelated changes, secrets, or debug artifacts are included
<!-- DOD:END -->
