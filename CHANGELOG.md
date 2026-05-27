# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- `invo config --validate` boolean flag. Config management is now organised
  under nested subcommands (TASK-8). Scripts using the old form will fail with
  a Clap parse error.

- `actions: [{ type: copy | move, ... }]` config shape. File-system actions
  now use a single `type: file` discriminator with a `kind: copy | move`
  field (TASK-77). The old shape will fail strict validation with
  `unknown variant`. Pre-release schema change — no migration shim.

### Changed

- `actions: [{ type: file, kind: copy | move, ... }]` is the new YAML shape
  for filesystem actions. Internally backed by a `PostAction` trait so each
  action variant carries its own dispatch (TASK-77). Migration:

  ```yaml
  # Before
  - type: copy
    destination: archive/
  - type: move
    destination: trash/

  # After
  - type: file
    kind: copy
    destination: archive/
  - type: file
    kind: move
    destination: trash/
  ```


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

- `processing.parallelism` config field (TASK-29). Default `1` preserves
  the previous sequential behavior. Values > 1 run up to N extractions
  concurrently and emit each result as soon as it completes — completion
  order, not input order. Each record's `file.path` / `file.hash` fields
  still self-identify which input it came from. Single-file runs ignore
  the setting. Implementation uses `futures::StreamExt::buffer_unordered`
  over a single shared `LlmContext`, so the reqwest connection pool is
  also shared across the parallel workers.

- Destination-collision detection for parallel post-processing actions
  (TASK-81, decision-4). When two workers' file/export templates resolve
  to the same path, the first writer wins; the second logs a WARN and
  skips. For Move actions, the second worker also preserves its source
  PDF (no delete on skip). Hook actions are not tracked because their
  destinations are user-script-defined.

- Config doctor warnings for `processing.parallelism` (TASK-79):
  - `parallelism: 0` → warns that it's clamped to 1 at runtime; set it
    to 1 explicitly to document intent.
  - `parallelism > 32` → warns that hosted providers' rate limits
    likely throttle retries above this threshold.

- Migration-pointing parse error for old action YAML (TASK-82). When a
  config still uses the pre-TASK-77 `type: copy` or `type: move` shape,
  the parse error now includes the new `type: file` + `kind: ...`
  example inline instead of relying on serde's generic "unknown variant"
  message.
