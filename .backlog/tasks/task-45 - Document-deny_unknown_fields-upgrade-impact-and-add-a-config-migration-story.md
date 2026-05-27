---
id: TASK-45
title: Document deny_unknown_fields upgrade impact and add a config migration story
status: To Do
assignee: []
created_date: '2026-05-27 00:08'
labels:
  - code-review
  - contract
  - docs
dependencies:
  - TASK-25
references:
  - src/config.rs
priority: high
ordinal: 47000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
TASK-25 added #[serde(deny_unknown_fields)] to every struct in src/config.rs. Users with the old default_config.yaml template (which included processing:, logging:, and other future blocks) now hard-fail on upgrade. config doctor cannot help because parse must succeed first. There is currently no CHANGELOG, version bump, or upgrade guide. Reviewer: api-contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CHANGELOG (or equivalent release notes) explains the breaking change and lists removed/restricted keys
- [ ] #2 config doctor (or a separate doctor pre-parse) prints a helpful 'remove this key' tip when the parse error mentions a known-removed field
- [ ] #3 README documents how an existing user should migrate their config
<!-- AC:END -->
