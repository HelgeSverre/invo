use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub(crate) const MIN_MEANINGFUL_TEXT_LENGTH: usize = 50;
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

/// Extract text from PDF bytes.
pub fn extract_text(bytes: &[u8]) -> Result<PdfExtractionResult> {
    let file_hash = short_sha256(bytes);

    let text = pdf_extract::extract_text_from_mem(bytes)
        .context("Failed to extract text from PDF bytes")?;
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

// Short identifier prefix of a SHA-256 over the PDF bytes. Today this is a
// 16-char lowercase hex prefix (64 bits — fine for human-scale collision
// avoidance; revisit when TASK-32 wires it into dedup, where birthday
// collisions matter at ~4B documents). We slice by char (not byte) so a
// future refactor to base64 or any non-ASCII encoding cannot panic on a
// non-char-boundary index.
fn short_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = format!("{:x}", hasher.finalize());
    hash.chars().take(SHORT_HASH_LEN).collect()
}

/// Count pages in a PDF (best effort)
fn count_pages(bytes: &[u8]) -> usize {
    String::from_utf8_lossy(bytes)
        .matches("/Type /Page")
        .count()
        .max(1)
}

/// Check whether bytes start with the PDF magic header.
pub fn is_pdf_bytes(bytes: &[u8]) -> bool {
    bytes.starts_with(PDF_MAGIC)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pdf_magic_bytes() {
        assert!(is_pdf_bytes(b"%PDF-1.7\n"));
        assert!(!is_pdf_bytes(b"not a pdf"));
    }

    #[test]
    fn short_sha256_is_16_lowercase_hex_chars() {
        let hash = short_sha256(b"%PDF-1.7\n");
        assert_eq!(hash.len(), SHORT_HASH_LEN);
        assert_eq!(hash.chars().count(), SHORT_HASH_LEN);
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "expected lowercase hex, got {hash:?}"
        );
    }

    #[test]
    fn short_sha256_is_deterministic() {
        assert_eq!(short_sha256(b"abc"), short_sha256(b"abc"));
        assert_ne!(short_sha256(b"abc"), short_sha256(b"abd"));
    }
}
