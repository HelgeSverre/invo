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

**Zsh (macOS/Linux):**
```bash
# For Homebrew-installed Zsh (recommended on macOS):
mkdir -p ~/.zsh/completions
invo completions --shell zsh > ~/.zsh/completions/_invo

# For system Zsh (check your $fpath):
invo completions --shell zsh > ${fpath[1]}/_invo

# Then add to your .zshrc:
fpath=(~/.zsh/completions $fpath)
autoload -U compinit && compinit
```

**Bash (macOS/Linux):**
```bash
# System-wide (may require sudo):
invo completions --shell bash > /usr/local/etc/bash_completion.d/invo

# User-local (recommended):
mkdir -p ~/.local/share/bash-completion/completions
invo completions --shell bash > ~/.local/share/bash-completion/completions/invo

# Add to your .bashrc/.bash_profile:
if [ -f ~/.local/share/bash-completion/completions/invo ]; then
    source ~/.local/share/bash-completion/completions/invo
fi
```

**Fish (macOS/Linux):**
```bash
invo completions --shell fish > ~/.config/fish/completions/invo.fish
```

**Detection script (recommended):**
```bash
# Detects the right location automatically
case $SHELL in
  */zsh)
    mkdir -p ~/.zsh/completions
    invo completions --shell zsh > ~/.zsh/completions/_invo
    echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
    echo 'autoload -U compinit && compinit' >> ~/.zshrc
    ;;
  */bash)
    mkdir -p ~/.local/share/bash-completion/completions
    invo completions --shell bash > ~/.local/share/bash-completion/completions/invo
    echo '[ -f ~/.local/share/bash-completion/completions/invo ] && source ~/.local/share/bash-completion/completions/invo' >> ~/.bashrc
    ;;
  */fish)
    invo completions --shell fish > ~/.config/fish/completions/invo.fish
    ;;
  *)
    echo "Unsupported shell: $SHELL"
    ;;
esac
echo "Completions installed! Restart your shell or run 'exec $SHELL'"
```

After installation, restart your shell or run `exec zsh`/`exec bash` to load the completions.

Requires Rust 1.85+ (rig-core 0.37 uses edition 2024) and, for the default provider, a running [Ollama](https://ollama.com) instance. Provider calls are implemented through [`rig-core`](https://crates.io/crates/rig-core).

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
| `invo config doctor [--json]` | Report config issues such as removed action types or missing OCR credentials |
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

See [`config.yaml`](./config.yaml) in the repo root for the active starter config written by `invo init`. Every key listed below is active at runtime; unknown keys are rejected at load time by strict validation.

| Config key | Status | Runtime effect |
| ---------- | ------ | -------------- |
| `identity.company`, `identity.name`, `identity.email`, `identity.address`, `identity.vat_number`, `identity.aliases` | Active | Added to the extraction prompt as buyer context so the model does not mistake you for the vendor |
| `llm.providers[].name`, `enabled`, `model`, `base_url`, `api_key_env` | Active | Selects and configures the LLM provider |
| `llm.extraction.temperature`, `max_tokens`, `retry_attempts` | Active | Passed into Rig completion requests and retry handling |
| `llm.extraction.max_pdf_text_chars` | Active | Cap on raw PDF text length passed to the LLM (default 32768) — defends against prompt injection blow-ups |
| `vendors[].name`, `patterns`, `currency`, `category`, `hints` | Active | Added to the extraction prompt as known vendor context |
| `output.format` | Active | Sets default `json`, `yaml`, or `csv` output |
| `processing.ocr.mode` (`none` \| `same` \| `mistral` \| `llm`), `processing.ocr.model`, `processing.ocr.api_key_env`, `processing.ocr.name`, `processing.ocr.base_url` | Active | OCR fallback for scanned PDFs — see "OCR fallback" below |
| `actions[]` (`copy` \| `move` \| `export` \| `hook`) | Active | Post-processing — see "Post-processing actions" below |

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

## OCR fallback

When native PDF text extraction yields very little text (<50 characters — typical for image-only or scanned PDFs), invo can fall back to a vision/document-model API to recover the text, then feed it into the normal extraction pipeline.

| Backend | PDF inline? | Native deps | Cargo feature | Default model |
| ------- | ----------- | ----------- | ------------- | ------------- |
| Mistral (`/v1/ocr`) | yes | none | — | `mistral-ocr-latest` |
| Anthropic (`/v1/messages` + `document` block) | yes | none | — | `claude-3-5-sonnet-latest` |
| OpenAI (`/v1/responses` + `input_file`) | yes | none | — | `gpt-4o` |
| Ollama (vision model + image content) | no — client-side rasterization | PDFium | `--features ollama-ocr` | `llama3.2-vision` |

Three of four backends are cross-platform with **zero native dependencies** — the PDF bytes are sent to a hosted API inline. Ollama vision models accept images only, so its branch is opt-in behind a Cargo feature that pulls in PDFium for rasterization.

The four config modes:

```yaml
# Off (default) — scanned PDFs error out with an actionable hint.
processing:
  ocr:
    mode: none
```

```yaml
# Mistral OCR endpoint — only needs an API key.
processing:
  ocr:
    mode: mistral
    # model: mistral-ocr-latest         # optional
    # api_key_env: MISTRAL_API_KEY      # optional, this is the default
```

```yaml
# Reuse the extractor provider/model. Works for mistral, anthropic, openai
# extractors (no rasterization), and ollama when built with --features ollama-ocr.
processing:
  ocr:
    mode: same
```

```yaml
# Explicit provider config.
processing:
  ocr:
    mode: llm
    name: anthropic               # mistral | anthropic | openai | ollama
    model: claude-3-5-sonnet-latest
    api_key_env: ANTHROPIC_API_KEY
    # base_url: ...               # ollama only
```

When OCR runs, `extraction.ocr_used` flips to `true` in the output.

**Limits & cost (per backend):**

- **Mistral**: 50 MB / 1000 pages per request. ~$0.70 per million OCR tokens (~1000 pages per USD) — see [Mistral pricing](https://mistral.ai/pricing). Mistral's Batch API is ~2× cheaper if you can tolerate async/queued delivery.
- **Anthropic**: 32 MB / 100 pages per request (via `document` content block). Standard input-token pricing for the model you pick.
- **OpenAI**: 32 MB per request (via Responses API `input_file`). Standard input-token pricing for the model you pick.
- **Ollama**: runs locally — no per-call cost. Latency depends on your hardware and the vision model. Quality varies; `llama3.2-vision` works for clean scans, less reliable for noisy/rotated pages.

**Enabling Ollama OCR:**

```bash
# Install PDFium for your platform:
brew install pdfium                                        # macOS
apt install libpdfium-dev                                  # Debian/Ubuntu
# Windows: download pdfium.dll from https://github.com/bblanchon/pdfium-binaries

# Rebuild invo with the feature:
cargo install --git https://github.com/HelgeSverre/invo --features ollama-ocr

# Pull a vision model and configure:
ollama pull llama3.2-vision
```

Without the feature, `mode: llm` / `mode: same` with Ollama returns an actionable rebuild error — the other three OCR backends still work in the default build.

## How it works

```text
   receipt.pdf  →  pdf_extract  →  LLM + schema prompt  →  Invoice JSON
                       text             provider              parsed
```

The [`pdf_extract`](https://crates.io/crates/pdf-extract) crate parses text out of the PDF. That text, together with a schema-aware system prompt, is sent to the configured provider. The model returns JSON constrained to the `Invoice` schema. invo parses the response — lenient on number and date formats, so currency symbols and comma decimals are handled — and prints the wrapped result. For image-only or scanned PDFs where `pdf_extract` returns nothing useful, the [OCR fallback](#ocr-fallback) routes the PDF bytes through a vision/document model (Mistral in v1) before extraction continues.

## Post-processing actions

Configured actions run after each successful extraction, in config order. Disabled actions are skipped; per-action failures log and continue, except for hook actions with `on_error: abort` which halt that file's processing.

Templates in `destination`, `filename`, and hook `args` substitute these placeholders: `{vendor}`, `{date}` (YYYY-MM-DD), `{year}`, `{month}`, `{day}`, `{invoice_number}` (empty when missing), `{currency}`, `{total}`, `{hash}`, `{original_name}`, `{stem}` (original name without `.pdf`). Path separators (`/`, `\`, `:`) inside expanded filename values are replaced with `_`.

```yaml
actions:
  # Copy the source PDF to an organized archive.
  - type: copy
    destination: "~/Documents/Receipts/{year}/{month}"
    filename: "{date}-{vendor}-{invoice_number}.pdf"
    create_dirs: true

  # Move (delete original after the copy succeeds).
  - type: move
    destination: "/tmp/invo-processed"
    create_dirs: true

  # Export the extracted JSON next to the source.
  - type: export
    format: json
    destination: "~/Documents/Receipts/{year}"
    filename: "{stem}.json"

  # Shell hook with env vars set: INVO_VENDOR, INVO_TOTAL, INVO_CURRENCY,
  # INVO_DATE, INVO_INVOICE_NUMBER, INVO_PATH, INVO_HASH, INVO_ORIGINAL_NAME.
  - type: hook
    enabled: true
    command: "/usr/local/bin/notify-send"
    args: ["Invoice processed", "{vendor} {total} {currency}"]
    on_error: continue       # or `abort` to halt this file on failure
```

`notify` action was removed — replace with a `hook` that calls your platform's notifier (`notify-send` on Linux, `osascript` / `terminal-notifier` on macOS).

## Trust model

> **PDF text is attacker-controllable. Do NOT auto-pay from invo output.**
>
> An invoice/receipt PDF can be authored by anyone — including the vendor whose totals you're trying to verify. The text inside the PDF flows into an LLM prompt, and LLMs are known to follow instructions hidden in their inputs (prompt injection). A crafted PDF can attempt to coerce the model into reporting an attacker-friendly vendor name or total.
>
> invo applies two defensive mitigations: (1) the PDF text region is wrapped in delimiters carrying a per-request 16-char nonce so an attacker cannot reliably close the delimiter, and (2) text is capped at `llm.extraction.max_pdf_text_chars` (default 32768) to bound prompt size. These are mitigations, not guarantees — a sufficiently clever prompt-injection-trained PDF may still steer the model.
>
> Use invo output as a structured-data starting point that a human reviews before money moves. Don't wire it directly into payment automation.

## Limitations

- **OCR off by default.** Set `processing.ocr.mode` to enable; the default build supports Mistral, Anthropic, and OpenAI without extra deps. Ollama OCR requires `--features ollama-ocr` (PDFium).
- **Serial processing.** Files are extracted one at a time.
- **CSV output is flat.** Line items, tax, and vendor metadata are dropped.

## License

MIT. See [`LICENSE`](./LICENSE).
