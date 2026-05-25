---
id: TASK-2
title: Rename root config template to config.yaml
status: Done
assignee:
  - '@codex'
created_date: '2026-05-25 12:57'
updated_date: '2026-05-25 13:25'
labels:
  - config
  - docs
dependencies: []
references:
  - REVIEW-CLI.md
modified_files:
  - config.yaml
  - src/main.rs
  - src/config/mod.rs
  - README.md
  - REVIEW-CLI.md
priority: medium
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Rename the root config template from default_config.yaml to config.yaml and update runtime/docs references.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 invo init embeds and writes the renamed root template.
- [x] #2 README and review/docs links no longer reference default_config.yaml.
- [x] #3 No stale default_config names remain except intentionally historical notes.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rename the checked-in template file from default_config.yaml to config.yaml.
2. Update the init command include_str! path and any documentation references to the new name.
3. Search for stale default_config references and keep only intentional historical notes if needed.
4. Run cargo test/build and CLI init/help smoke checks that do not overwrite the user config.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Renamed the root template to config.yaml, updated init to embed the renamed file, renamed the default-path helper to user_config_path to remove stale default_config naming, and updated README/REVIEW references. Verified the renamed template validates and init writes byte-identical config.yaml into a temporary HOME.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
TASK-2 renames the repo config template from default_config.yaml to config.yaml and updates runtime/docs references. Verified with cargo test, cargo build, target/debug/invo -c config.yaml config --validate, and a temporary-HOME init smoke that compares the generated file to config.yaml.
<!-- SECTION:FINAL_SUMMARY:END -->
