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

/// Count pages in a PDF (best effort).
///
/// Prefers `lopdf` because it correctly handles compressed object streams
/// (PDF 1.5+) where the `/Type /Page` literal does not appear in the raw
/// bytes. Falls back to a delimiter-aware byte scan when lopdf cannot parse
/// the document — the scan is allocation-free and rejects the `/Type /Pages`
/// catalog entry to avoid the +1 overcount the previous string-based pass had.
fn count_pages(bytes: &[u8]) -> usize {
    if let Ok(doc) = lopdf::Document::load_mem(bytes) {
        let pages = doc.get_pages().len();
        if pages > 0 {
            return pages;
        }
    }
    count_pages_by_scan(bytes).max(1)
}

fn count_pages_by_scan(bytes: &[u8]) -> usize {
    let needle = b"/Type /Page";
    bytes
        .windows(needle.len())
        .enumerate()
        .filter(|(start, window)| {
            *window == needle
                && bytes
                    .get(start + needle.len())
                    .is_some_and(|b| !is_pdf_name_continuation(*b))
        })
        .count()
}

// True for bytes that can appear inside a PDF /Name token (per PDF 32000-1
// section 7.3.5). If the byte directly after `/Type /Page` is one of these,
// the match is actually `/Pages` (or `/PageLayout`, etc.) — not a real page.
fn is_pdf_name_continuation(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'-'
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

    #[test]
    fn count_pages_by_scan_rejects_pages_catalog_false_positive() {
        // /Type /Pages is the page catalog, not a leaf page. The previous
        // string-matches() pass counted it as +1 page on every PDF.
        let bytes = b"3 0 obj <</Type /Pages /Kids [4 0 R] /Count 1>> endobj\n\
                      4 0 obj <</Type /Page /Parent 3 0 R>> endobj\n";

        assert_eq!(count_pages_by_scan(bytes), 1);
    }

    #[test]
    fn count_pages_by_scan_counts_uncompressed_multi_page_layout() {
        // Two page objects with a /Type /Pages catalog. The catalog must be
        // rejected; only the two leaf pages should count.
        let bytes = b"1 0 obj <</Type /Catalog /Pages 2 0 R>> endobj\n\
                      2 0 obj <</Type /Pages /Kids [3 0 R 4 0 R] /Count 2>> endobj\n\
                      3 0 obj <</Type /Page /Parent 2 0 R>> endobj\n\
                      4 0 obj <</Type /Page /Parent 2 0 R>> endobj\n";

        assert_eq!(count_pages_by_scan(bytes), 2);
    }

    #[test]
    fn count_pages_single_page_real_sample() {
        let bytes = std::fs::read("samples/misc/2025.01.25 - Laravel Vapor - Jan.pdf")
            .expect("sample pdf should be readable");

        assert_eq!(count_pages(&bytes), 1);
    }

    #[test]
    fn count_pages_compressed_multi_page_real_sample() {
        // PDF 1.5 documents commonly use compressed object streams, where the
        // /Type /Page literal never appears in raw bytes. lopdf decompresses
        // and reports the real count; the legacy string match returned 1.
        let bytes = std::fs::read("samples/misc/2025.11.30 - GoogleCloudEMEALimited.pdf")
            .expect("sample pdf should be readable");

        assert_eq!(count_pages(&bytes), 2);
    }
}
