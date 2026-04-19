use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

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
    let bytes = std::fs::read(path)
        .with_context(|| format!("Failed to read PDF file: {:?}", path))?;

    // Calculate file hash
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());
    let file_hash = hash[..16].to_string(); // Short hash

    // Try native text extraction first
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .with_context(|| format!("Failed to extract text from PDF: {:?}", path))?;

    // Count pages (approximate by splitting on form feeds or using heuristics)
    let page_count = count_pages(&bytes);

    // Check if extraction was successful (has meaningful text)
    let cleaned_text = text.trim();
    if cleaned_text.len() < 50 {
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

/// Count pages in a PDF (best effort)
fn count_pages(bytes: &[u8]) -> usize {
    // Simple heuristic: count /Page objects
    // This is a rough estimate, but works for most PDFs
    let content = String::from_utf8_lossy(bytes);
    let count = content.matches("/Type /Page").count();
    if count > 0 {
        count
    } else {
        // Fallback: assume 1 page
        1
    }
}

/// Check if a file is a valid PDF (by extension + magic bytes).
pub fn is_pdf(path: &Path) -> bool {
    let Some(ext) = path.extension() else { return false };
    if ext.to_string_lossy().to_lowercase() != "pdf" {
        return false;
    }

    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else { return false };
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).is_ok() && &magic == b"%PDF"
}
