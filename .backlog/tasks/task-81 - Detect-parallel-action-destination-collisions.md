---
id: TASK-81
title: Detect parallel-action destination collisions
status: Done
assignee: []
created_date: '2026-05-27 16:04'
updated_date: '2026-05-27 16:15'
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
- [x] #1 ADR documenting the chosen behavior added under .backlog/decisions/
- [x] #2 When two parallel workers resolve to the same Copy/Move/Export destination, behavior follows the ADR
- [x] #3 When a Move worker is skipped due to collision, the source PDF is NOT removed
- [x] #4 Test covers the collision path with parallelism: 2 and a template that produces identical destinations
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decision-4 documents the chosen policy: first writer wins, second worker logs WARN and skips, Move never deletes source on skip. Implementation: new public DestinationRegistry type (Arc<Mutex<HashSet<PathBuf>>>) and new_destination_registry constructor in src/actions.rs. PostAction::run gained a &DestinationRegistry parameter. FileAction (Copy/Move) and ExportAction call try_claim_destination before writing; on collision they tracing::warn and return Ok without writing. HookAction ignores the registry (user-script destinations are invisible). main.rs creates one registry per batch and Arc::clones it into each worker closure; single-file mode passes a fresh registry trivially. Two new collision tests: copy_action_with_colliding_destination_is_skipped_and_first_wins, move_action_with_colliding_destination_skips_and_preserves_source. README and CHANGELOG updated. Test count: 135+16 -> 137+16.
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
