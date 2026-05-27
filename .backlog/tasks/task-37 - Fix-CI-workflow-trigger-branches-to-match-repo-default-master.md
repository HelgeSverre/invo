---
id: TASK-37
title: Fix CI workflow trigger branches to match repo default (master)
status: Done
assignee: []
created_date: '2026-05-27 00:07'
labels:
  - code-review
  - ci
  - bug
dependencies: []
references:
  - .github/workflows/ci.yml
priority: high
ordinal: 39000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ci.yml triggers on branches: [main] but the repo default branch is master. The workflow never fires on pushes to master or PRs targeting master, so all CI checks added on this branch are effectively dead. Reviewer: agent-native.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ci.yml push trigger lists branches: [master]
- [x] #2 ci.yml pull_request trigger lists branches: [master]
- [ ] #3 A test commit to master shows CI running
<!-- AC:END -->
