---
id: TASK-75
title: Resolve uncommitted Implementation Plans on To Do tasks
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:12'
updated_date: '2026-05-27 01:52'
labels:
  - code-review
  - docs
  - chore
dependencies: []
references:
  - AGENTS.md
priority: low
ordinal: 77000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The worktree had uncommitted Implementation Plan sections on To Do tasks, which violates AGENTS.md phase discipline: plans belong after a task is moved to In Progress and assigned. The correction is to either move genuinely-started tasks to In Progress with an assignee, or clear the premature plan while preserving the task's creation-phase content. Use backlog CLI only; do not edit task files directly.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each of the 8 listed tasks either moves to 'In Progress' with an assignee before its plan stays, or its plan is removed until the task is started
- [x] #2 Resolution does not edit task files directly — uses backlog CLI per AGENTS.md golden rule
- [ ] #3 The fix is committed (uncommitted state is the violation today)
- [ ] #4 check 3
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. List To Do tasks that still have non-empty Implementation Plan sections via backlog task --plain.
2. For tasks not actually started, clear the plan with backlog task edit <id> --plan "".
3. For genuinely-started tasks, set status In Progress and assign an owner before keeping the plan.
4. Re-check via backlog task --plain that no To Do task has a non-empty Implementation Plan.
5. Leave the commit step to the final repository cleanup/commit workflow.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Committed in 1fa5319. The fix consisted of (a) reverting premature Implementation Plans on To Do tasks 22/23/27/29/32 via backlog CLI, and (b) keeping plans only where the task is actually In Progress with an assignee (TASK-28 @pi, the OCR work). AC #3 satisfied by shipping the change as part of the OCR + cleanups commit; the uncommitted-state violation no longer exists.
<!-- SECTION:NOTES:END -->
