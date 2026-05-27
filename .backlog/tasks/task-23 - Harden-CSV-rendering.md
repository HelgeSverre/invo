---
id: TASK-23
title: Harden CSV rendering
status: Done
assignee: []
created_date: '2026-05-25 15:31'
updated_date: '2026-05-27 12:00'
labels:
  - output
  - cli
dependencies: []
modified_files:
  - src/output.rs
  - Cargo.toml
priority: medium
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make CSV output robust for quoted, comma-containing, and multiline string values without changing the current CSV columns.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All string fields emitted to CSV are escaped consistently.
- [x] #2 Vendor and invoice number values containing quotes, commas, or newlines render as valid CSV.
- [x] #3 The CSV column set remains vendor,date,total,currency,invoice_number.
- [x] #4 Tests cover CSV escaping edge cases.
<!-- AC:END -->
