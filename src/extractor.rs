use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;
use std::time::Instant;

use crate::config::{Config, ProviderName};
use crate::invoice::{ExtractionMeta, ExtractionResult, FileInfo};
use crate::llm::extract_invoice_data;
use crate::pdf::extract_text;
use crate::prompt::{extraction_system_prompt, extraction_user_message};

pub async fn extract_invoice(
    path: &Path,
    config: &Config,
    provider_override: Option<ProviderName>,
) -> Result<ExtractionResult> {
    tracing::info!("Processing: {:?}", path);
    let start = Instant::now();

    let pdf_result = extract_text(path).context("Failed to extract text from PDF")?;
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
            path: path.to_string_lossy().to_string(),
            original_name: path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
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
