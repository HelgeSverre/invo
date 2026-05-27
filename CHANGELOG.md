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
