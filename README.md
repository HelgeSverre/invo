# invo

A Rust CLI that extracts structured data from PDF receipts and invoices using LLMs, locally by default.

## Install

```bash
cargo install --git https://github.com/HelgeSverre/invo
```

From a clone, for contributors:

```bash
cargo install --path .
```

## Shell Completions

invo supports shell completions for Bash, Zsh, and Fish:

**Zsh:**
```bash
invo completions --shell zsh > /usr/local/share/zsh/site-functions/_invo
```

**Bash:**
```bash
invo completions --shell bash > /etc/bash_completion.d/invo
# or for local installation:
invo completions --shell bash > ~/.local/share/bash-completion/completions/invo
```

**Fish:**
```bash
invo completions --shell fish > ~/.config/fish/completions/invo.fish
```

After installation, restart your shell or run `exec zsh`/`exec bash` to load the completions.

Requires Rust 1.70+ and, for the default provider, a running [Ollama](https://ollama.com) instance. Provider calls are implemented through [`rig-core`](https://crates.io/crates/rig-core).

## Quick start

```bash
ollama pull qwen2.5:1.5b
cargo install --git https://github.com/HelgeSverre/invo
invo init
invo samples/claude/Receipt-2238-2456-7832.pdf   # from a clone of this repo
invo path/to/your-receipt.pdf                    # or any PDF of your own
```

Optional: add buyer identity context (`identity.company`, `identity.name`, `identity.email`, `identity.address`, `identity.vat_number`, or `identity.aliases`) when your own company/name appears on receipts and the model might confuse you with the vendor.

That's it. Everything below is reference.

## Output

invo emits JSON by default (`--format json`), with a stable top-level shape. A single result is pretty-printed for readability unless you pass `--json`. Multiple JSON results are emitted as compact NDJSON, one object per line, so batch output can be parsed as a stream. Use `--json-array` when you need one JSON array instead.

```json
{
  "file": {
    "path": "samples/claude/Receipt-2238-2456-7832.pdf",
    "original_name": "Receipt-2238-2456-7832.pdf",
    "hash": "6f1e...a3c9",
    "processed_at": "2026-04-19T10:04:11Z"
  },
  "invoice": {
    "vendor": "Anthropic, PBC",
    "invoice_date": "2026-03-13",
    "invoice_number": "2238-2456-7832",
    "total_amount": 125.00,
    "subtotal": 100.00,
    "tax_amount": 25.00,
    "tax_rate": 0.25,
    "currency": "USD",
    "buyer_name": "Liseth Solutions AS",
    "items": [
      { "description": "API credits — top-up", "quantity": 1, "unit_price": 100.00, "total": 100.00 },
      { "description": "VAT (25%)",            "quantity": 1, "unit_price": 25.00,  "total": 25.00  }
    ],
    "category": "software",
    "confidence": 0.94
  },
  "extraction": {
    "provider": "ollama",
    "model": "qwen2.5:1.5b",
    "ocr_used": false,
    "processing_time_ms": 1412
  }
}
```

**Required fields** on `invoice`:

| Field          | Type   |
| -------------- | ------ |
| `vendor`       | string |
| `invoice_date` | string (YYYY-MM-DD) |
| `total_amount` | number |
| `currency`     | string (ISO 4217) |

**Optional fields** on `invoice`:

| Field | Type |
| ----- | ---- |
| `invoice_number`, `due_date` | string, string (YYYY-MM-DD) |
| `subtotal`, `tax_amount`, `tax_rate`, `discount_amount` | number |
| `vendor_website`, `vendor_email`, `vendor_address` | string |
| `payment_method`, `card_last_four` | string |
| `items[].sku`, `items[].description`, `items[].quantity`, `items[].unit_price`, `items[].total` | string, string, number, number, number |
| `category`, `description` | string |
| `buyer_name` | string |
| `confidence` | number (0.0–1.0) |

**Wrapper fields**:

| Field | Type |
| ----- | ---- |
| `file.path`, `file.original_name`, `file.hash`, `file.processed_at` | string |
| `extraction.provider`, `extraction.model` | string |
| `extraction.ocr_used` | boolean |
| `extraction.processing_time_ms` | number |
| `identity_match` | object or null |

`--format yaml` emits the same shape. `--format csv` emits only `vendor,date,total,currency,invoice_number` — lossy by design.

**stdout/stderr contract:** successful extraction data is written to stdout only. Diagnostics, progress, warnings, validation errors, and logs are written to stderr.

## Commands & flags

| Flag | Values | Purpose |
| ---- | ------ | ------- |
| `-f, --format` | `json`, `yaml`, `csv` | Output format (overrides config) |
| `--json` | — | Machine-readable JSON where supported; extraction batches are compact NDJSON |
| `--json-array` | — | Emit one JSON array containing all successful results |
| `--pretty` | — | Pretty-print JSON for single-result human display |
| `-c, --config` | path | Use a specific config file |
| `--files-from` | path or `-` | Read newline-delimited PDF paths/globs from a file or stdin |
| `--provider` | `ollama`, `openai`, `anthropic`, `mistral` | Force a provider for this run |
| `-v, --verbose` | repeatable (`-v`, `-vv`) | Debug / trace logging |
| `-h, --help` | — | Show help |

| Subcommand | Purpose |
| ---------- | ------- |
| `invo init [--force]` | Write a starter config to `~/.config/invo/config.yaml`, or to `-c/--config <path>` when supplied |
| `invo config path` | Print the config path invo will use |
| `invo config show --effective [--json]` | Show effective config after defaults are applied |
| `invo config validate [--json]` | Strictly validate config, or report that built-in defaults are in use when the file is missing |
| `invo config doctor [--json]` | Report parsed-only/inactive config keys |
| `invo config schema --json` | Emit the JSON schema for config.yaml |
| `invo agent-context --json` | Emit structured CLI/config context for agents and scripts |

```bash
# Glob over a directory of receipts as compact NDJSON
invo "receipts/2026/**/*.pdf" --json > out.ndjson
```

```bash
# Pipe PDF bytes from stdin; stdin metadata is reported as stdin.pdf
invo - < receipt.pdf
```

```bash
# Read newline-delimited PDF paths or globs from a file
printf '%s\n' 'receipts/january.pdf' 'receipts/february.pdf' > receipts.txt
invo --files-from receipts.txt --json > out.ndjson
```

```bash
# Emit one parseable JSON array for a batch
invo "receipts/2026/**/*.pdf" --json-array > receipts.json
```

```bash
# Validate a specific config; --config works before or after config subcommands
invo config validate -c config.yaml --json
```

```bash
# Show effective config and CLI context for agents/scripts
invo config show --effective --json
invo agent-context --json
```

For agents/scripts, prefer `--json` for extraction streams, `--json-array` when a single JSON document is easier, `invo config validate --json` before editing config, and `invo agent-context --json` for command/config discovery.

```bash
# Force OpenAI with trace logging. The openai provider entry may be disabled in YAML;
# --provider selects it for this run, but api_key_env must name an environment variable.
export OPENAI_API_KEY=sk-...
invo receipt.pdf --provider openai -vv
```

## Configuration

The config lives at `~/.config/invo/config.yaml` (override with `-c <file>`). Setup is currently non-interactive: generate a starter file with `invo init`, or write to a custom destination with `invo -c ./config.yaml init`.

```yaml
identity:
  company: "Liseth Solutions AS"
  email: "helge@example.com"
  vat_number: "NO123456789MVA"

llm:
  providers:
    - name: ollama
      enabled: true
      model: "qwen2.5:1.5b"
      base_url: "http://localhost:11434"
  extraction:
    temperature: 0.1
    max_tokens: 2000

vendors:
  - name: "Anthropic"
    patterns: ["Anthropic, PBC", "anthropic.com"]
    currency: "USD"
    category: "software"
    hints: "Anthropic receipts often show Anthropic, PBC as the legal vendor."

output:
  format: json
```

See [`config.yaml`](./config.yaml) in the repo root for the active starter config written by `invo init`. The parser accepts some future fields for compatibility, but `invo init` only writes settings that affect runtime behavior today.

| Config key | Status | Runtime effect |
| ---------- | ------ | -------------- |
| `identity.company`, `identity.name`, `identity.email`, `identity.address`, `identity.vat_number`, `identity.aliases` | Active | Added to the extraction prompt as buyer context so the model does not mistake you for the vendor |
| `llm.providers[].name`, `enabled`, `model`, `base_url`, `api_key_env` | Active | Selects and configures the LLM provider |
| `llm.extraction.temperature`, `max_tokens`, `retry_attempts` | Active | Passed into Rig completion requests and retry handling |
| `vendors[].name`, `patterns`, `currency`, `category`, `hints` | Active | Added to the extraction prompt as known vendor context |
| `output.format` | Active | Sets default `json`, `yaml`, or `csv` output |
| `llm.providers[].model_path`, `ignored_vendors`, `actions`, `processing`, `logging`, `output.include_hash`, `output.schema_version` | Future | Parsed only; not written by `invo init`; `invo config doctor` reports these when present |

## Providers

| Provider | Status | Default model | Auth | Base URL |
| -------- | ------ | ------------- | ---- | -------- |
| `ollama` | **Working**, default | `qwen2.5:1.5b` | none | `http://localhost:11434` |
| `openai` | **Working** | user-set in config | `OPENAI_API_KEY` | `https://api.openai.com` |
| `anthropic` | **Working** | user-set in config | `ANTHROPIC_API_KEY` | `https://api.anthropic.com` |
| `mistral` | **Working** | user-set in config | `MISTRAL_API_KEY` | `https://api.mistral.ai` |

invo uses Rig-backed completion providers. It selects the first provider with `enabled: true` in config. Override per-run with `--provider <name>` to select any configured provider entry by name, even if that entry has `enabled: false`. The override does not invent missing provider settings: for OpenAI, the selected provider still needs `model` and `api_key_env`, and the named environment variable must be set. If you force an unimplemented provider, invo errors instead of silently picking another.

### Provider Configuration

Each provider supports the following configuration fields:

- `name`: Provider identifier (`ollama`, `openai`, `anthropic`, `mistral`)
- `enabled`: Whether this provider is active by default
- `model`: Model name to use
- `base_url`: Optional custom base URL (overrides provider defaults)
- `api_key_env`: Environment variable name for API key (required for cloud providers)

### Base URL Support

All providers support custom `base_url` configuration:

```yaml
llm:
  providers:
    - name: anthropic
      enabled: true
      model: claude-3-opus-20240229
      base_url: "https://custom-anthropic-proxy.example.com"
      api_key_env: ANTHROPIC_API_KEY
    - name: mistral
      enabled: false
      model: mistral-large-latest
      base_url: "https://mistral-local:8080"
      api_key_env: MISTRAL_API_KEY
```

When `base_url` is omitted, providers use their standard endpoints:
- Ollama: `http://localhost:11434`
- OpenAI: `https://api.openai.com`
- Anthropic: `https://api.anthropic.com`
- Mistral: `https://api.mistral.ai`

## How it works

```text
   receipt.pdf  →  pdf_extract  →  LLM + schema prompt  →  Invoice JSON
                       text             provider              parsed
```

The [`pdf_extract`](https://crates.io/crates/pdf-extract) crate parses text out of the PDF. That text, together with a schema-aware system prompt, is sent to the configured provider. The model returns JSON constrained to the `Invoice` schema. invo parses the response — lenient on number and date formats, so currency symbols and comma decimals are handled — and prints the wrapped result. Because extraction relies on `pdf_extract`, image-only PDFs produce no text and will error out.

## Limitations

- **No OCR.** Image-only or scanned PDFs produce empty text and error out.
- **Serial processing.** Files are extracted one at a time.
- **Anthropic and Mistral providers are not implemented.** The CLI accepts `--provider anthropic|mistral` but returns an error.
- **CSV output is flat.** Line items, tax, and vendor metadata are dropped.

## License

MIT. See [`LICENSE`](./LICENSE).
