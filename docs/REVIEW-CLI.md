# CLI and Config UX Review

Date: 2026-05-25

Scope reviewed:

- `docs/7 Principles for Agent-Friendly CLIs.txt`
- `docs/10 Principles for Agent-Native CLIs.txt`
- `docs/Design exploration with AI agents.txt`
- `src/`
- `README.md`
- `config.yaml`
- live CLI help, validation, init, error, and one extraction path

## Summary

`invo` has a good base for both humans and agents: it is a small CLI, non-interactive by default, has typed Clap enums, emits structured extraction data, and keeps config optional. The main problem is that the current interface promises more machine-friendliness and config behavior than it actually delivers.

The highest-impact fix is to make stdout sacred: extraction data only on stdout, diagnostics only on stderr. The current logger writes warnings/info lines to stdout, so `--format json`, `--format yaml`, and `--format csv` are not reliably parseable.

The second issue is config trust. `config.yaml` exposes many fields that parse but do not affect runtime behavior. That makes the config look powerful, but it weakens discoverability because users cannot tell which knobs matter.

Scores:

| Area | Score | Rationale |
| --- | ---: | --- |
| Agent friendliness | 5/10 | Non-interactive and structured formats exist, but stdout is polluted, multi-file JSON is not a single parseable stream, no structured introspection exists, and config validation misses typos. |
| Human CLI friendliness | 6/10 | Quick start is simple and `invo init` exists, but help lacks examples, `config` can silently do nothing, `init` ignores `--config`, and there is no setup/doctor flow. |
| Config discoverability | 4/10 | Identity is useful but hidden in a generated YAML file; many advertised config fields are inert; validation reports missing config as valid. |
| Documentation fit | 6/10 | README is direct and honest in places, but provider override, NDJSON, OCR, retry, and inactive config surfaces need sharper alignment with runtime behavior. |

## Confirmed Findings

### HIGH: Runtime logs are emitted to stdout, corrupting data output

Where:

- `src/main.rs:68`
- `src/main.rs:93`
- `src/pipeline.rs:19`
- `src/pipeline.rs:67`

Evidence:

- `target/debug/invo README.md >/tmp/invo-stdout.txt 2>/tmp/invo-stderr.txt`
- stdout contained the warning log: `WARN invo: Skipping "README.md" - not a PDF file`
- stderr contained the user error: `No PDF files found matching the specified patterns.`
- A successful extraction also printed `INFO` log lines before the JSON payload.

Why it matters:

The reviewed CLI articles treat clean stdout/stderr separation as a core agent requirement. The current behavior means `invo receipt.pdf --format json | jq .` can fail because stdout is not only JSON.

Recommendation:

- Configure tracing to write to stderr explicitly.
- Consider defaulting runtime logs to `warn` or `error` unless `-v` is passed.
- Keep all diagnostics, progress, and hints on stderr.
- Add a CLI regression test that runs a sample command with redirected stdout/stderr and asserts stdout starts with parseable JSON/YAML/CSV only.

### HIGH: Multi-file JSON output is not actually JSON or NDJSON

Where:

- `src/main.rs:97`
- `src/main.rs:170`
- `README.md:118`

Issue:

The main loop prints one pretty JSON object per file. For more than one file, stdout becomes adjacent pretty-printed JSON documents, not a JSON array and not NDJSON. The README example writes globbed output to `out.ndjson`, but the emitted JSON is multi-line.

Why it matters:

Agents and scripts need a stable stream contract. Adjacent pretty JSON is difficult to parse correctly and violates the README example.

Recommendation:

- Add `--json` as the agent-friendly default alias for machine JSON.
- Make batch JSON one of:
  - `--json` emits NDJSON, one compact JSON object per line.
  - `--json-array` emits a single array.
  - `--pretty` only affects single-result human display.
- Update README examples to match the real stream format.

### HIGH: `config.yaml` advertises fields that are parsed but unused

Where:

- `config.yaml:48`
- `config.yaml:57`
- `config.yaml:60`
- `config.yaml:68`
- `config.yaml:78`
- `src/config/schema.rs:19`
- `src/config/schema.rs:211`
- `src/config/schema.rs:244`
- `src/config/schema.rs:320`
- `src/config/schema.rs:388`
- `README.md:158`

Issue:

The default config includes `ignored_vendors`, `output.include_hash`, `output.schema_version`, `actions`, `processing.parallelism`, `processing.ocr`, `processing.deduplication`, and `logging`, but runtime does not use these sections today.

Specific contradictions:

- README says there is no OCR, while default config enables OCR.
- README says there are no retries, while config has `retry_attempts`.
- `include_hash: false` would not remove `file.hash`.
- `logging.level` is ignored; CLI verbosity/env controls logging.
- `actions` do not run after extraction.

Why it matters:

This makes config look more mature than the tool. A human will set fields expecting behavior. An agent will preserve or modify fields that do nothing.

Recommendation:

- Make `config.yaml` contain only active settings.
- Move future/inert settings to `docs/config-reference.md` or `examples/full_config.future.yaml`, clearly marked as not active.
- Alternatively add `invo config doctor` that reports inactive keys in the current config.

### MEDIUM: `config --validate` can report success when no config exists

Where:

- `src/config/mod.rs:17`
- `src/config/mod.rs:21`
- `src/main.rs:131`

Evidence:

- `env HOME=/tmp/invo-review-empty-home target/debug/invo config --validate`
- Output: `Config is valid!`

Issue:

`load_config` returns `Config::default()` when the file is absent, so validation confirms defaults rather than validating an existing file.

Why it matters:

For humans, this makes it harder to discover whether a config file exists. For agents, it hides state and can lead to false assumptions.

Recommendation:

- Change validation output to include path and existence:
  - `Config file: ~/.config/invo/config.yaml`
  - `Status: missing; using built-in defaults`
- Add `invo config path`, `invo config show --effective --json`, and `invo config doctor`.

### MEDIUM: Unknown config keys are silently accepted

Where:

- `src/config/schema.rs:8`

Evidence:

- `target/debug/invo -c <(printf 'identitty: {}\n') config --validate`
- Output: `Config is valid!`

Issue:

Serde accepts unknown fields by default. A typo like `identitty` or `outptu` passes validation but has no effect.

Why it matters:

This is a bad failure mode for both humans and agents. It looks successful while silently dropping the user's intent.

Recommendation:

- Add `#[serde(deny_unknown_fields)]` to config structs once the active config surface is trimmed.
- If future sections must remain parseable, explicitly nest them under `experimental` or mark them as ignored in `config doctor`.

### MEDIUM: `invo config` exits successfully with no output

Where:

- `src/main.rs:51`
- `src/main.rs:131`

Evidence:

- `target/debug/invo config`
- Exit code 0, no output.

Issue:

The only config action is a `--validate` boolean. Without it, the command silently does nothing.

Recommendation:

Use subcommands instead of a boolean:

```text
invo config path
invo config show [--effective] [--json]
invo config validate [--strict] [--json]
invo config doctor [--json]
invo config schema --json
```

### MEDIUM: `init` ignores `--config`

Where:

- `src/main.rs:33`
- `src/main.rs:112`
- `src/main.rs:115`

Evidence:

- `env HOME=/tmp/invo-review-init-home target/debug/invo -c /tmp/invo-review-custom.yaml init`
- Created `/tmp/invo-review-init-home/.config/invo/config.yaml`
- Did not create `/tmp/invo-review-custom.yaml`

Issue:

Users reasonably expect `-c/--config` to choose the init destination. The global option is parsed but ignored by `Init`.

Recommendation:

- Respect `cli.config` in `init`.
- Or remove `--config` from commands where it is irrelevant.
- Prefer `invo config init --path <file>` if config management becomes a command group.

### MEDIUM: Global flags are not accepted after subcommands

Where:

- `src/main.rs:33`
- `src/main.rs:46`

Evidence:

- `target/debug/invo config --validate -c config.yaml`
- Clap error: `unexpected argument '-c' found`
- `target/debug/invo -c config.yaml config --validate` works.

Issue:

This is technically valid Clap behavior, but it is a friction point. Humans often put `--config` next to the subcommand it affects.

Recommendation:

- Make `--config` a global Clap arg if it should apply everywhere.
- Add examples in help either way.

### MEDIUM: `--provider` is not a true force override

Where:

- `src/llm/mod.rs:16`
- `src/llm/mod.rs:20`
- `README.md:109`
- `README.md:123`
- `README.md:169`

Issue:

The code skips disabled providers before checking the override. So `--provider openai` only works if OpenAI is already enabled in config. That conflicts with the wording "Force a provider for this run."

Why it matters:

Provider switching is exactly where users need the CLI to be explicit and self-correcting. The current error, `No enabled provider named OpenAI found in config`, does not tell the user that OpenAI must exist and be enabled in config.

Recommendation:

- If `--provider` is supplied, select that provider by name even if `enabled: false`.
- Or rename behavior to `--only-provider` and update docs.
- Best UX: support `--provider openai --model gpt-4o-mini` without requiring YAML edits.

### MEDIUM: Identity config is useful but under-explained and partially inert

Where:

- `config.yaml:4`
- `src/config/schema.rs:35`
- `src/config/schema.rs:57`
- `src/extraction/prompt.rs:12`
- `src/extraction/prompt.rs:26`

Issue:

Identity is the strongest optional config feature because it helps the LLM distinguish buyer from vendor. But several declared fields do not affect the prompt:

- `address` is parsed but not included in the prompt.
- `vat_number` is parsed but not included in the prompt.
- `is_configured` ignores `address` and `vat_number`, so a config with only VAT/address will not enable identity context.

Recommendation:

- Document identity as "optional buyer context" and show one concrete before/after.
- Include all identity fields in the prompt or remove inactive fields from the default config.
- Add `invo config doctor` warnings:
  - `identity.vat_number is set but currently not used`
  - `identity.address is set but currently not used`

### MEDIUM: Vendor config contains fields that do not affect extraction

Where:

- `config.yaml:39`
- `src/config/schema.rs:177`
- `src/extraction/prompt.rs:29`

Issue:

`Vendor.currency` and `Vendor.hints` parse, but the prompt only uses name, patterns, and category.

Recommendation:

- Either include `currency` and `hints` in the prompt or remove/comment them from the active sample.
- Prefer explicit config docs:
  - Active: `name`, `patterns`, `category`
  - Parsed but not active: none

### LOW: Help is present but not example-driven

Where:

- `src/main.rs:17`
- live `target/debug/invo --help`
- docs principle: `docs/7 Principles for Agent-Friendly CLIs.txt:229`
- docs principle: `docs/10 Principles for Agent-Native CLIs.txt:246`

Issue:

Top-level help lists commands and flags, but there are no examples for the high-probability flows:

- extract one PDF
- extract a glob as NDJSON
- validate effective config
- initialize config at a custom path
- use OpenAI

Recommendation:

Use Clap `after_help` or long_about examples. Keep examples short and copy-pasteable.

### LOW: Data input is file-only

Where:

- `src/main.rs:25`
- `src/main.rs:142`

Issue:

The CLI accepts file paths/globs only. That is fine for humans, but agents often pipe bytes or file lists.

Recommendation:

- Support `-` as a PDF stdin path.
- Add `--files-from <path>` and allow `--files-from -`.
- For stdin PDF bytes, require `--filename` or synthesize `stdin.pdf` for metadata.

## What Works Well

- No command currently prompts or hangs, so the tool is safe for non-interactive agent execution.
- Clap enum validation is good. Invalid providers enumerate valid values.
- The default provider path works without API keys if Ollama is running.
- The README is unusually honest that many full config keys are not wired yet.
- The actual extraction prompt uses identity and vendors in a way that matches the intended feature direction.
- `config.yaml` is readable and close to what a human would want to edit by hand.

## Recommended CLI Shape

Keep the primary extraction path simple:

```text
invo [OPTIONS] <FILE>...
```

Add agent- and human-friendly affordances around it:

```text
invo receipt.pdf --json
invo "receipts/**/*.pdf" --json              # NDJSON, compact one object per line
invo "receipts/**/*.pdf" --json-array        # one JSON array
invo receipt.pdf --pretty                    # human-oriented pretty JSON
invo -                                       # PDF bytes from stdin
invo --files-from receipts.txt --json
```

Config commands:

```text
invo setup                                   # interactive TTY wizard for humans
invo setup --no-input --provider ollama      # non-interactive bootstrap
invo setup --provider openai --model gpt-4o-mini --identity-company "Acme AS"

invo config path
invo config init [--path <file>] [--force]
invo config show --effective --json
invo config validate --strict --json
invo config doctor
invo config schema --json
```

Agent introspection:

```text
invo agent-context --json
```

Suggested `agent-context` payload:

```json
{
  "schema_version": 1,
  "commands": {
    "extract": {
      "usage": "invo [OPTIONS] <FILE>...",
      "outputs": ["json", "yaml", "csv"],
      "stdout_contract": "data only",
      "stderr_contract": "diagnostics only"
    },
    "config": {
      "subcommands": ["path", "show", "validate", "doctor", "schema"]
    }
  },
  "config": {
    "path": "~/.config/invo/config.yaml",
    "exists": true,
    "active_keys": [
      "identity.company",
      "identity.name",
      "identity.email",
      "identity.aliases",
      "llm.providers",
      "llm.extraction.temperature",
      "llm.extraction.max_tokens",
      "vendors.name",
      "vendors.patterns",
      "vendors.category",
      "output.format"
    ]
  }
}
```

## Recommended Setup Wizard

Add `invo setup`, but do not make extraction itself interactive.

Rules:

- Only prompt when stdin is a TTY.
- Never prompt in the extraction command.
- Support `--no-input` so agents can prove it will not hang.
- Put all setup hints on stderr.
- Do not make config required for extraction.

Human flow:

```text
$ invo setup
Config path: ~/.config/invo/config.yaml

? Provider:  Ollama local
? Ollama URL: http://localhost:11434
? Model: qwen2.5:1.5b
? Add buyer identity context? yes
? Company/name shown on your receipts: Liseth Solutions AS
? Add aliases? liseth.no, Helge Sverre

Wrote ~/.config/invo/config.yaml
Run: invo config doctor
```

Non-interactive equivalent:

```text
invo setup --no-input \
  --provider ollama \
  --model qwen2.5:1.5b \
  --identity-company "Liseth Solutions AS" \
  --identity-alias "Helge Sverre"
```

First-run hint:

- If no config exists and stderr is a TTY, print one concise hint to stderr:
  - `Optional: run "invo setup" to add buyer identity context and improve vendor detection.`
- Do not print this hint to stdout.
- Suppress the hint in non-TTY mode unless `--verbose` is set.

## Recommended `config.yaml` Shape

Keep the generated default config small and honest:

```yaml
# invo configuration file
# Location: ~/.config/invo/config.yaml
#
# This file is optional. invo works without it.
# Identity helps the model identify YOU/the buyer so it does not mistake you
# for the vendor on receipts.

identity:
  # company: "Your Company AS"
  # name: "Your Name"
  # email: "you@example.com"
  aliases: []

llm:
  providers:
    - name: ollama
      enabled: true
      model: "qwen2.5:1.5b"
      base_url: "http://localhost:11434"

  extraction:
    temperature: 0.1
    max_tokens: 2000

vendors: []
  # - name: "Anthropic"
  #   patterns: ["Anthropic, PBC", "anthropic.com"]
  #   category: "software"

output:
  format: json
```

Move future features to separate examples until wired:

```text
examples/config.full.future.yaml
docs/config-reference.md
```

In docs, use a status table:

| Config key | Status | Runtime effect |
| --- | --- | --- |
| `identity.company` | active | Included in extraction prompt as buyer context |
| `identity.aliases` | active | Included in extraction prompt as buyer aliases |
| `identity.vat_number` | inactive today | Parsed only |
| `vendors.patterns` | active | Included in known vendor prompt |
| `vendors.currency` | inactive today | Parsed only |
| `actions` | inactive today | Parsed only |
| `processing.ocr` | inactive today | Parsed only |
| `logging` | inactive today | Parsed only |

## Documentation Updates

Recommended README changes:

- Replace `out.ndjson` example unless output becomes compact NDJSON.
- Clarify that `--provider openai` currently requires an enabled OpenAI provider in config.
- Remove or mark OpenAI "force" example until CLI flags can supply provider/model without YAML.
- Move inactive config fields out of the main config sample.
- Add "Optional identity context" near quick start:
  - "If your receipts often contain your company/name, run `invo setup` or add `identity.company` so the extractor can distinguish buyer from vendor."
- Document stdout/stderr contract:
  - stdout: extraction data only
  - stderr: diagnostics, progress, warnings, setup hints
- Add a "For agents/scripts" block:
  - use `--json`
  - put options before subcommands until global flag handling changes
  - use `invo config show --effective --json`
  - use `invo agent-context --json`

## Prioritized Roadmap

1. Fix stdout/stderr separation.
2. Define batch output contract: NDJSON or array.
3. Trim generated config to active keys.
4. Make `config validate` honest about missing files and unknown fields.
5. Respect `--config` in `init`, or move init under `config init`.
6. Add `config path/show/doctor/schema`.
7. Add `--json` as a conventional alias and keep `--format` for compatibility.
8. Add `invo setup` with TTY-only prompts and non-interactive flags.
9. Add `agent-context --json`.
10. Add CLI snapshot tests for help, stdout/stderr split, validation errors, and config doctor.

## Verification Commands Run

```bash
git status --short
git diff --stat
cargo run -- --help
cargo run -- config --help
cargo run -- init --help
target/debug/invo -c config.yaml config --validate
target/debug/invo config --validate -c config.yaml
target/debug/invo config
env HOME=/tmp/invo-review-empty-home target/debug/invo config --validate
env HOME=/tmp/invo-review-init-home target/debug/invo -c /tmp/invo-review-custom.yaml init
target/debug/invo README.md >/tmp/invo-stdout.txt 2>/tmp/invo-stderr.txt
target/debug/invo -c <(printf 'identitty: {}\n') config --validate
target/debug/invo -c <(printf 'output:\n  format: xml\n') config --validate
target/debug/invo --provider giraffe
```

One sample extraction was also run with a temporary OpenAI config to verify success-path stdout behavior. It confirmed that info logs are printed before the JSON payload.
