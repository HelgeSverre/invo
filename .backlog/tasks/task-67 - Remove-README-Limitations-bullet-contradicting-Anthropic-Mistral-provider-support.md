---
id: TASK-67
title: >-
  Remove README 'Limitations' bullet contradicting Anthropic/Mistral provider
  support
status: Done
assignee: []
created_date: '2026-05-27 00:11'
updated_date: '2026-05-27 00:43'
labels:
  - code-review
  - docs
  - chore
dependencies:
  - TASK-30
references:
  - README.md
modified_files:
  - README.md
priority: low
ordinal: 69000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
README.md:337 'Limitations' section still says 'Anthropic and Mistral providers are not implemented. The CLI accepts --provider anthropic|mistral but returns an error.' The providers table earlier in README marks both Working, and TASK-30 is Done. The contradictory bullet was missed when TASK-30 landed. Reviewers: correctness, learnings-researcher.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The contradictory 'not implemented' bullet is removed
- [x] #2 Providers table remains the single source of truth
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Resolved in the current README. The obsolete limitation claiming Anthropic and Mistral providers are unimplemented is gone, while the providers table remains the source of truth for provider support.
<!-- SECTION:FINAL_SUMMARY:END -->
