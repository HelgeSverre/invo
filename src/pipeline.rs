use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;
use std::time::Instant;

use crate::config::Config;
use crate::extraction::{
    extraction_system_prompt, extraction_user_message, invoice_schema, ExtractionMeta,
    ExtractionResult, FileInfo, Invoice,
};
use crate::llm::{LlmProvider, LlmRequest};
use crate::pdf::extract_text;

pub async fn extract_invoice(
    path: &Path,
    config: &Config,
    provider: &dyn LlmProvider,
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

    let request = LlmRequest {
        system_prompt: extraction_system_prompt(&config.identity, &config.vendors),
        user_message: extraction_user_message(&pdf_result.text),
        json_schema: Some(invoice_schema().clone()),
        temperature: config.llm.extraction.temperature,
        max_tokens: config.llm.extraction.max_tokens,
    };

    tracing::debug!("Sending to LLM provider: {}", provider.name());
    let response = provider.complete(request).await?;
    tracing::trace!("LLM response:\n{}", response.content);

    let invoice: Invoice = serde_json::from_str(&response.content)
        .with_context(|| format!("Failed to parse LLM response:\n{}", response.content))?;

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
        invoice,
        extraction: ExtractionMeta {
            provider: response.provider,
            model: response.model,
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
