---
id: TASK-53
title: Reject combining --files-from - with positional '-' (stdin double-read)
status: Done
assignee:
  - '@claude'
created_date: '2026-05-27 00:09'
labels:
  - code-review
  - bug
  - cli
dependencies: []
references:
  - src/main.rs
priority: medium
ordinal: 55000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/main.rs:564 input_patterns() calls read_files_from() (which reads stdin if path='-') BEFORE expand_inputs() iterates cli.files. The stdin_count>1 guard at main.rs:595-601 only checks patterns after expansion. 'cat receipt.pdf | invo --files-from - -' drains the PDF binary into the file list as garbage path strings, then expand_inputs hits '-' in cli.files and gets 0 bytes from stdin and bails 'stdin input was empty'. Reviewers: correctness, adversarial.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 When cli.files_from == Some("-") and any positional argument is '-', the CLI rejects the combination at parse time with a clear error
- [ ] #2 Alternatively, stdin consumption is tracked across both readers so only one consumer touches stdin (chose AC #1 instead)
- [x] #3 Integration test pins the rejection (or shared-stdin) behavior
<!-- AC:END -->
