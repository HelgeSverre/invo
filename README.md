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

Requires Rust 1.70+ and, for the default provider, a running [Ollama](https://ollama.com) instance.

## Quick start

```bash
ollama pull qwen2.5:1.5b
cargo install --git https://github.com/HelgeSverre/invo
invo init
invo samples/claude/Receipt-2238-2456-7832.pdf   # from a clone of this repo
invo path/to/your-receipt.pdf                    # or any PDF of your own
```

That's it. Everything below is reference.

## Output

invo emits JSON by default (`--format json`), with a stable top-level shape.

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

## Commands & flags

| Flag | Values | Purpose |
| ---- | ------ | ------- |
| `-f, --format` | `json`, `yaml`, `csv` | Output format (overrides config) |
| `-c, --config` | path | Use a specific config file |
| `--provider` | `ollama`, `openai`, `anthropic`, `mistral` | Force a provider for this run |
| `-v, --verbose` | repeatable (`-v`, `-vv`) | Debug / trace logging |
| `-h, --help` | — | Show help |

| Subcommand | Purpose |
| ---------- | ------- |
| `invo init` | Write a starter config to `~/.config/invo/config.yaml` |
| `invo config validate` | Parse the config and report whether it is valid |
| `invo config show` | Print the effective merged config |
| `invo config path` | Print the resolved config file path |
| `invo config doctor` | Diagnose common config issues |
| `invo config schema` | Print the JSON schema for the config |

```bash
# Glob over a directory of receipts
invo "receipts/2026/**/*.pdf" > out.ndjson
```

```bash
# Force OpenAI with trace logging
export OPENAI_API_KEY=sk-...
invo receipt.pdf --provider openai -vv
```

## Configuration

The config lives at `~/.config/invo/config.yaml` (override with `-c <file>`). Generate one with `invo init`.

```yaml
identity:
  company: "Liseth Solutions AS"
  email: "helge@example.com"

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

output:
  format: json
```

See [`default_config.yaml`](./default_config.yaml) in the repo root for the full surface written by `invo init`. Only the keys shown above take effect today; the rest parse successfully but are not yet wired into the runtime.

## Providers

| Provider | Status | Default model | Auth |
| -------- | ------ | ------------- | ---- |
| `ollama` | **Working**, default | `qwen2.5:1.5b` | none (local at `http://localhost:11434`) |
| `openai` | **Working** | user-set in config | `OPENAI_API_KEY` |
| `anthropic` | Not implemented | — | — |
| `mistral` | Not implemented | — | — |

invo uses the first provider with `enabled: true` in config. Override per-run with `--provider <name>`; if you force an unimplemented one, invo errors instead of silently picking another.

## How it works

```text
   receipt.pdf  →  pdf_extract  →  LLM + schema prompt  →  Invoice JSON
                       text             provider              parsed
```

The [`pdf_extract`](https://crates.io/crates/pdf-extract) crate parses text out of the PDF. That text, together with a schema-aware system prompt, is sent to the configured provider. The model returns JSON constrained to the `Invoice` schema. invo parses the response — lenient on number and date formats, so currency symbols and comma decimals are handled — and prints the wrapped result. Because extraction relies on `pdf_extract`, image-only PDFs produce no text and will error out.

## Limitations

- **No OCR.** Image-only or scanned PDFs produce empty text and error out.
- **Serial processing.** Files are extracted one at a time.
- **No retries.** A transient LLM failure fails the file.
- **Anthropic and Mistral providers are not implemented.** The CLI accepts `--provider anthropic|mistral` but returns an error.
- **CSV output is flat.** Line items, tax, and vendor metadata are dropped.

## License

MIT. See [`LICENSE`](./LICENSE).
