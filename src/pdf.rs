use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

const MIN_MEANINGFUL_TEXT_LENGTH: usize = 50;
const PDF_MAGIC: &[u8; 4] = b"%PDF";
const SHORT_HASH_LEN: usize = 16;

/// Result of PDF text extraction
#[derive(Debug)]
pub struct PdfExtractionResult {
    pub text: String,
    pub page_count: usize,
    pub file_hash: String,
    pub ocr_used: bool,
}

/// Extract text from a PDF file
pub fn extract_text(path: &Path) -> Result<PdfExtractionResult> {
    let bytes =
        std::fs::read(path).with_context(|| format!("Failed to read PDF file: {:?}", path))?;

    extract_text_from_bytes(&bytes, &format!("{:?}", path))
}

/// Extract text from PDF bytes, using source_name for diagnostics only.
pub fn extract_text_from_bytes(bytes: &[u8], source_name: &str) -> Result<PdfExtractionResult> {
    let file_hash = short_sha256(bytes);

    let text = pdf_extract::extract_text_from_mem(bytes)
        .with_context(|| format!("Failed to extract text from PDF: {}", source_name))?;
    let page_count = count_pages(bytes);

    let cleaned_text = text.trim();
    if cleaned_text.len() < MIN_MEANINGFUL_TEXT_LENGTH {
        tracing::warn!(
            "PDF text extraction yielded very little text ({} chars), OCR may be needed",
            cleaned_text.len()
        );
    }

    Ok(PdfExtractionResult {
        text: cleaned_text.to_string(),
        page_count,
        file_hash,
        ocr_used: false,
    })
}

fn short_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = format!("{:x}", hasher.finalize());
    hash[..SHORT_HASH_LEN].to_string()
}

/// Count pages in a PDF (best effort)
fn count_pages(bytes: &[u8]) -> usize {
    String::from_utf8_lossy(bytes)
        .matches("/Type /Page")
        .count()
        .max(1)
}

/// Check if a file is a valid PDF (by extension + magic bytes).
pub fn is_pdf(path: &Path) -> bool {
    if !has_pdf_extension(path) {
        return false;
    }

    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).is_ok() && is_pdf_bytes(&magic)
}

/// Check whether bytes start with the PDF magic header.
pub fn is_pdf_bytes(bytes: &[u8]) -> bool {
    bytes.starts_with(PDF_MAGIC)
}

fn has_pdf_extension(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.to_string_lossy().to_lowercase() == "pdf")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pdf_magic_bytes() {
        assert!(is_pdf_bytes(b"%PDF-1.7\n"));
        assert!(!is_pdf_bytes(b"not a pdf"));
    }
}
