use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::config::{Config, ProviderName};
use crate::invoice::{ExtractionMeta, ExtractionResult, FileInfo};
use crate::llm::extract_invoice_data;
use crate::pdf::{extract_text, extract_text_from_bytes};
use crate::prompt::{extraction_system_prompt, extraction_user_message};

const STDIN_METADATA_NAME: &str = "stdin.pdf";

#[derive(Debug, Clone)]
pub struct InvoiceInput {
    pub path: String,
    pub original_name: String,
    source: InvoiceInputSource,
}

#[derive(Debug, Clone)]
enum InvoiceInputSource {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

impl InvoiceInput {
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            path: path.to_string_lossy().to_string(),
            original_name: original_name(&path),
            source: InvoiceInputSource::Path(path),
        }
    }

    pub fn stdin(bytes: Vec<u8>) -> Self {
        Self {
            path: STDIN_METADATA_NAME.to_string(),
            original_name: STDIN_METADATA_NAME.to_string(),
            source: InvoiceInputSource::Bytes(bytes),
        }
    }
}

pub async fn extract_invoice(
    input: &InvoiceInput,
    config: &Config,
    provider_override: Option<ProviderName>,
) -> Result<ExtractionResult> {
    tracing::info!("Processing: {}", input.path);
    let start = Instant::now();

    let pdf_result = match &input.source {
        InvoiceInputSource::Path(path) => {
            extract_text(path).context("Failed to extract text from PDF")?
        }
        InvoiceInputSource::Bytes(bytes) => extract_text_from_bytes(bytes, &input.path)
            .context("Failed to extract text from PDF")?,
    };
    if pdf_result.text.is_empty() {
        anyhow::bail!("No text extracted from PDF");
    }
    tracing::debug!(
        "Extracted {} chars from {} pages",
        pdf_result.text.len(),
        pdf_result.page_count
    );

    let llm_result = extract_invoice_data(
        config,
        provider_override,
        extraction_system_prompt(&config.identity, &config.vendors),
        extraction_user_message(&pdf_result.text),
    )
    .await?;

    let result = ExtractionResult {
        file: FileInfo {
            path: input.path.clone(),
            original_name: input.original_name.clone(),
            hash: pdf_result.file_hash,
            processed_at: Utc::now().to_rfc3339(),
        },
        invoice: llm_result.invoice,
        extraction: ExtractionMeta {
            provider: llm_result.provider,
            model: llm_result.model,
            ocr_used: pdf_result.ocr_used,
            processing_time_ms: start.elapsed().as_millis() as u64,
        },
        identity_match: None,
    };

    tracing::info!(
        "Extracted invoice from {} - {} {} on {}",
        result.invoice.vendor,
        result.invoice.total_amount,
        result.invoice.currency,
        result.invoice.invoice_date
    );

    Ok(result)
}

fn original_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdin_input_uses_deterministic_metadata() {
        let input = InvoiceInput::stdin(b"%PDF-1.7\n".to_vec());

        assert_eq!(input.path, "stdin.pdf");
        assert_eq!(input.original_name, "stdin.pdf");
    }

    #[test]
    fn path_input_uses_file_name_metadata() {
        let input = InvoiceInput::from_path(PathBuf::from("samples/example.pdf"));

        assert_eq!(input.path, "samples/example.pdf");
        assert_eq!(input.original_name, "example.pdf");
    }
}
