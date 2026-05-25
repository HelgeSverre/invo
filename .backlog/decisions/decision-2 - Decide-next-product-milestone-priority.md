---
id: decision-2
title: Decide next product milestone priority
date: '2026-05-25 15:31'
status: Proposed
---
## Context

The initial cleanup and CLI hardening work is complete: existing tasks are Done, and the backlog now contains both near-term maintenance work and larger product features.

Current follow-up tasks include:

- TASK-19 — Make Rust quality checks green and add CI
- TASK-20 — Add missing MIT LICENSE file
- TASK-21 — Remove unused direct dependencies
- TASK-22 — Make batch extraction failures script-safe
- TASK-23 — Harden CSV rendering
- TASK-24 — Split CLI plumbing out of main.rs
- TASK-25 — Reduce config validation drift risk
- TASK-26 — Clarify or implement provider base_url behavior
- TASK-27 — Add sample PDF regression checks
- TASK-28 — Add OCR fallback for scanned PDFs
- TASK-29 — Add parallel batch processing
- TASK-30 — Implement Anthropic and Mistral providers
- TASK-31 — Implement post-processing actions
- TASK-32 — Implement deduplication using file hashes

The next milestone should be chosen deliberately because the backlog now mixes different kinds of work:

1. **Foundation/quality work** — CI, clippy, dependencies, license, tests.
2. **CLI reliability work** — batch failure semantics, CSV correctness, config/base_url clarity.
3. **Maintainability work** — splitting `main.rs`, reducing config validation drift.
4. **Product expansion** — OCR, parallelism, more providers, actions, deduplication.

The main decision is whether to prioritize developer confidence and script safety first, or move directly into user-visible extraction capabilities like OCR/provider expansion.

## Decision

Proposed milestone order:

### Milestone 1: Release hygiene and script safety

Goal: make the existing product safer to install, test, automate, and release before expanding scope.

Recommended tasks:

1. TASK-19 — Make Rust quality checks green and add CI
2. TASK-20 — Add missing MIT LICENSE file
3. TASK-21 — Remove unused direct dependencies
4. TASK-22 — Make batch extraction failures script-safe
5. TASK-23 — Harden CSV rendering
6. TASK-26 — Clarify or implement provider base_url behavior
7. TASK-27 — Add sample PDF regression checks

Why this first:

- CI and clippy provide a safety net for all later feature work.
- Script-safe failure semantics are critical for an agent-friendly CLI.
- CSV/base_url issues are small but user-visible correctness gaps.
- Sample PDF tests help future OCR/provider changes avoid regressions.

### Milestone 2: Maintainability pass

Goal: reduce friction before the next round of feature work.

Recommended tasks:

1. TASK-24 — Split CLI plumbing out of main.rs
2. TASK-25 — Reduce config validation drift risk
3. Decision-1 follow-up — resolve parsed-only config policy

Why second:

- `main.rs` is large enough that feature work will become harder if all CLI/config/input/output concerns stay together.
- Config validation drift risk grows as OCR, parallelism, providers, actions, and dedupe are implemented.
- This keeps the source tree flat while still giving modules clear responsibilities.

### Milestone 3: Extraction capability expansion

Goal: improve the range of invoices the tool can process.

Recommended order:

1. TASK-28 — Add OCR fallback for scanned PDFs
2. TASK-30 — Implement Anthropic and Mistral providers
3. TASK-29 — Add parallel batch processing

Why this order:

- OCR addresses a major limitation: scanned/image-only PDFs fail today.
- Provider expansion helps users who need stronger cloud models or already use Anthropic/Mistral.
- Parallelism is valuable after reliability semantics are settled, because concurrency complicates ordering and partial failure behavior.

### Milestone 4: Workflow automation

Goal: add post-extraction workflow features once extraction itself is reliable.

Recommended tasks:

1. TASK-32 — Implement deduplication using file hashes
2. TASK-31 — Implement post-processing actions

Why later:

- Deduplication and actions require careful semantics around skipped files, output records, hooks, side effects, and failures.
- They are valuable, but they should not be implemented before batch failure behavior and output contracts are fully settled.

## Consequences

Positive consequences:

- The next work is sequenced to reduce risk before adding scope.
- The CLI becomes safer for scripts and agents before adding more side-effecting features.
- Future product work gets better test and CI coverage.
- Refactoring is tied to a concrete need (`main.rs` size and config drift), not done prematurely.

Tradeoffs:

- User-visible feature work like OCR is delayed until release hygiene is done.
- Provider support for Anthropic/Mistral remains unavailable for at least one milestone.
- Some parsed-only config fields remain unresolved until the config policy decision is accepted and acted on.

Alternative considered:

- **Prioritize OCR immediately.** This would address a clear user limitation sooner, but it would land on top of weak CI, unclear batch failure behavior, and limited sample-based regression coverage.
- **Prioritize provider expansion immediately.** This would increase model choice, but it does not fix scanned PDFs, output reliability, or release confidence.
- **Prioritize refactoring first.** This would improve maintainability, but release hygiene and script correctness are more urgent because they affect every user and automation path.

Exit criteria for accepting this decision:

- The backlog is grouped or labeled according to these milestone buckets.
- Work starts with TASK-19 unless there is an urgent user-facing bug.
- New feature tasks should not jump ahead of Milestone 1 unless they are small, low-risk, and do not weaken CLI/script guarantees.
