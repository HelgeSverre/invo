---
id: TASK-36
title: Harden CSV rendering for formula injection and field-breaking content
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:06'
updated_date: '2026-05-27 00:49'
labels:
  - code-review
  - security
  - csv
  - bug
dependencies:
  - TASK-23
references:
  - src/output.rs
modified_files:
  - src/output.rs
  - Cargo.toml
priority: high
ordinal: 38000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
render_csv_results writes Invoice vendor, invoice_number, currency, and total_amount with hand-rolled escaping that misses formula-injection (=, +, -, @, tab, CR), does not escape embedded commas/newlines in unquoted columns, and emits NaN/Inf for invalid numeric content. A malicious PDF with vendor like '=cmd|/c calc!A0' executes a formula when the user opens the CSV in Excel. Reviewers: security, adversarial, agent-native, correctness. (Supersedes part of TASK-23.)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All Invoice fields are CSV-escaped: quote-wrapped, embedded quotes doubled
- [x] #2 Fields starting with =, +, -, @, tab, or CR are prefixed with a leading single quote
- [x] #3 Implementation uses the csv crate (or a tested equivalent) instead of hand-rolled csv_escape
- [x] #4 Regression tests cover formula-leading values, embedded commas, embedded newlines, embedded quotes, and NaN/Inf totals
<!-- AC:END -->
