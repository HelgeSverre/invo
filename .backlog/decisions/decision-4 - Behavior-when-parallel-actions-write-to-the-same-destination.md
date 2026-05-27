---
id: decision-4
title: Behavior when parallel actions write to the same destination
date: '2026-05-27 16:11'
status: accepted
---
## Context

TASK-29 introduced `processing.parallelism > 1`. When two extraction workers run `actions::run_actions` concurrently and their filename templates expand to the same filesystem destination, they race on writes. Concrete example: two invoices from the same vendor with a template `"{vendor}.pdf"` both resolve to `archive/Acme.pdf`. The second worker silently overwrites the first; neither result is reliably preserved.

Three candidate behaviors:

- **(a) Skip duplicates:** First writer wins. Second worker logs WARN and skips its file action. Extraction success counter unchanged (extractions still succeeded; only the redundant action was skipped).
- **(b) Abort batch:** Any detected collision propagates as an error and the batch exits with the partial-failure exit code (3).
- **(c) Warn-only:** Log loudly but allow the overwrite. Last writer wins.

Option (b) makes a *batch-wide* hard failure out of what is usually a misconfigured-template UX bug — it punishes the user's other files for one bad template. Option (c) preserves the data-loss bug; the warning informs but doesn't prevent it. Option (a) preserves the first writer's data, surfaces the issue loudly, and keeps the batch moving.

## Decision

**Accept option (a): skip duplicates, first writer wins, log WARN.**

- For Copy/Export actions: if the resolved destination is already claimed by another worker, log WARN with both the input path and the destination, and skip the write.
- For Move actions: the source PDF is **not removed** when the destination is already claimed. This is critical — without this rule, two workers with the same destination template would both try to delete different source files even though only one source's contents ended up at the destination. Skip-without-delete preserves the second file for the user to re-process with a better template.
- Hook actions are not tracked: they write via user-supplied scripts to arbitrary paths the framework cannot observe. Hook collisions remain the user's responsibility.

Detection: a shared `Arc<Mutex<HashSet<PathBuf>>>` registry created once per batch invocation. Workers attempt to `insert(dest)` before writing. The insert returns `true` if the destination is new (worker proceeds); `false` if already claimed (worker skips with WARN).

## Consequences

**Good:**

- Data loss prevented: the first invoice that lands at a colliding destination is preserved.
- Visible: the WARN message gives the user enough information to identify the bad template and fix it.
- Batch keeps moving: one user-config bug doesn't fail unrelated files.

**Trade-offs:**

- "First writer" is non-deterministic when parallelism > 1 — which input actually lands at a given destination depends on completion order. The WARN identifies which one was skipped, so the user can re-run after fixing the template.
- The skip is silent on stdout (data only); only stderr gets the WARN. Scripts that consume stdout don't see the collision directly, but the missing-file evidence + stderr summary make it discoverable.

**Out of scope (deferred):**

- Pre-flight conflict detection: walking the actions+inputs list before the batch starts and rejecting templates that can be shown to collide. Useful but expensive to compute precisely (depends on extracted vendor/invoice-number values that aren't known until extraction completes). Could be filed as a future task if user feedback warrants it.
- Hook coordination: same problem class but the framework can't see hook destinations. User's responsibility for now.
