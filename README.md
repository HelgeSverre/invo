# invo

CLI tool to extract structured data from PDF receipts and invoices using LLMs.

## Install

```bash
cargo install --path .
```

## Usage

```bash
invo init                           # create ~/.config/invo/config.yaml
invo receipt.pdf                    # extract a single PDF
invo "invoices/*.pdf"               # glob multiple files
invo receipt.pdf --format yaml      # json (default), yaml, or csv
invo receipt.pdf --provider openai  # override configured provider
```

## Providers

Configured in `~/.config/invo/config.yaml`. Tried in order until one succeeds.

- **ollama** (local, default: `qwen2.5:1.5b`)
- **openai** (`OPENAI_API_KEY`)
- **anthropic** (`ANTHROPIC_API_KEY`)

## Output

Structured JSON with vendor, date, total, currency, line items, and extraction metadata (provider, model, confidence, timing).
