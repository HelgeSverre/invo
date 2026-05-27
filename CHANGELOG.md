# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- `invo config --validate` boolean flag. Config management is now organised
  under nested subcommands (TASK-8). Scripts using the old form will fail with
  a Clap parse error.

### Changed

- Config management moved from a single `--validate` flag to discoverable
  subcommands. Migration mapping:

  | Old                       | New                  |
  | ------------------------- | -------------------- |
  | `invo config --validate`  | `invo config validate` |
  | _(no equivalent)_         | `invo config show`     |
  | _(no equivalent)_         | `invo config path`     |
  | _(no equivalent)_         | `invo config doctor`   |
  | _(no equivalent)_         | `invo config schema`   |

- README updated to reference the new subcommand form (TASK-46).

- Multi-file `--json` output is now **NDJSON** (one compact JSON object per
  line) instead of a sequence of pretty-printed JSON blobs (TASK-47). The
  old shape was not valid JSON either way, so consumers parsing it as a
  document already had to do something custom; the new shape is a stable
  streamable contract. Single-file `--json` output is unchanged (one
  pretty-printed object). Use `--json-array` if you need a single JSON
  array document for a batch.

  Migration example (Bash + jq, before and after a batch run):

  ```sh
  # Old: sequence of pretty JSON blobs — needed a custom parser.
  # New: NDJSON — each line is a complete JSON object.
  invo "receipts/**/*.pdf" --json | jq -c '.invoice.vendor + " " + (.invoice.total_amount|tostring)'

  # Or aggregate the stream into an array for tools that want one document:
  invo "receipts/**/*.pdf" --json | jq -s '.'
  ```

### Added

- Script-safe batch exit codes (TASK-22): `0` all succeeded, `2` zero
  successes, `3` partial success (>=1 succeeded AND >=1 failed). Single-file
  failures still propagate as anyhow errors. Any batch with failures prints a
  one-line summary to stderr: `Batch: N succeeded, M failed (of K files).`
