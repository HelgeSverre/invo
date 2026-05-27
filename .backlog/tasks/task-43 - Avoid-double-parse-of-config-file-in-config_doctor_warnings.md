---
id: TASK-43
title: Avoid double-parse of config file in config_doctor_warnings
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:08'
updated_date: '2026-05-27 00:49'
labels:
  - code-review
  - maintainability
  - config
  - refactor
dependencies: []
references:
  - src/config_commands.rs
  - src/config.rs
priority: high
ordinal: 45000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
config doctor now lives in src/config_commands.rs after the TASK-24 split. It still calls load_config_status (which reads and parses the typed Config) and then config_doctor_warnings re-reads the same file and parses raw serde_yaml::Value to detect present-but-inactive keys. The double parse and raw YAML traversal should be removed or justified.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 config_doctor_warnings moves into src/config.rs (or wherever Config lives) and accepts &Config (now in src/config.rs, takes &LoadedConfig so we can keep the "config file is missing" branch — interior reads loaded.config: &Config directly)
- [x] #2 Raw YAML probes are replaced with typed struct-field checks (match on OcrConfig variants; the legacy actions[].type = notify probe deleted because deny_unknown_fields rejects it at parse time)
- [x] #3 serde_yaml::{Mapping,Value} imports are removed from main.rs (and from src/config_commands.rs — the value_at + mapping_get helpers went with them)
<!-- AC:END -->
