use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use crate::pdf::is_pdf_bytes;

const STDIN_METADATA_NAME: &str = "stdin.pdf";

#[derive(Debug, Clone)]
pub struct InputOptions {
    pub files: Vec<PathBuf>,
    pub files_from: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct InvoiceInput {
    pub path: String,
    pub original_name: String,
    pub bytes: Vec<u8>,
}

// InvoiceInputSpec carries the bytes already read during validation, not a
// re-readable path. This closes a TOCTOU between magic-byte validation and
// later text extraction: a symlink swap or sync-rewrite between the two
// opens cannot substitute different content for what was approved.
#[derive(Debug, Clone)]
pub struct InvoiceInputSpec {
    pub path: String,
    pub original_name: String,
    bytes: Vec<u8>,
}

impl InvoiceInputSpec {
    fn from_path_with_bytes(path: &Path, bytes: Vec<u8>) -> Self {
        Self {
            path: path.to_string_lossy().to_string(),
            original_name: original_name(path),
            bytes,
        }
    }

    fn stdin(bytes: Vec<u8>) -> Self {
        Self {
            path: STDIN_METADATA_NAME.to_string(),
            original_name: STDIN_METADATA_NAME.to_string(),
            bytes,
        }
    }

    pub fn read(self) -> Result<InvoiceInput> {
        Ok(InvoiceInput {
            path: self.path,
            original_name: self.original_name,
            bytes: self.bytes,
        })
    }
}

pub fn collect_inputs(options: &InputOptions) -> Result<Vec<InvoiceInputSpec>> {
    let patterns = input_patterns(options)?;
    expand_inputs(&patterns)
}

fn input_patterns(options: &InputOptions) -> Result<Vec<PathBuf>> {
    let mut patterns = options.files.clone();

    if let Some(files_from) = &options.files_from {
        patterns.extend(read_files_from(files_from)?);
    }

    Ok(patterns)
}

fn read_files_from(path: &Path) -> Result<Vec<PathBuf>> {
    let contents = if is_stdin_marker(path) {
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut contents)
            .context("Failed to read newline-delimited file list from stdin")?;
        contents
    } else {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file list: {}", path.display()))?
    };

    Ok(parse_file_list(&contents))
}

fn parse_file_list(contents: &str) -> Vec<PathBuf> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(PathBuf::from)
        .collect()
}

fn expand_inputs(patterns: &[PathBuf]) -> Result<Vec<InvoiceInputSpec>> {
    let stdin_count = patterns
        .iter()
        .filter(|pattern| is_stdin_marker(pattern))
        .count();
    if stdin_count > 1 {
        bail!("stdin PDF input '-' can only be specified once");
    }

    let mut inputs = Vec::new();

    for pattern in patterns {
        if is_stdin_marker(pattern) {
            inputs.push(read_stdin_pdf_input()?);
            continue;
        }

        let pattern_str = pattern.to_string_lossy();

        if pattern_str.contains('*') || pattern_str.contains('?') {
            for entry in glob::glob(&pattern_str)? {
                let path = entry?;
                match read_pdf_if_valid(&path) {
                    Some(bytes) => {
                        inputs.push(InvoiceInputSpec::from_path_with_bytes(&path, bytes))
                    }
                    None => tracing::debug!("Skipping {:?} - not a PDF file", path),
                }
            }
        } else if pattern.exists() {
            match read_pdf_if_valid(pattern) {
                Some(bytes) => inputs.push(InvoiceInputSpec::from_path_with_bytes(pattern, bytes)),
                None => tracing::warn!("Skipping {:?} - not a PDF file", pattern),
            }
        } else {
            tracing::warn!("File not found: {:?}", pattern);
        }
    }

    Ok(inputs)
}

fn read_stdin_pdf_input() -> Result<InvoiceInputSpec> {
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut std::io::stdin(), &mut bytes)
        .context("Failed to read PDF bytes from stdin")?;

    if bytes.is_empty() {
        bail!("stdin input was empty. Pipe PDF bytes with `invo -`, or use --files-from <path> for newline-delimited file paths.");
    }

    if !is_pdf_bytes(&bytes) {
        bail!("stdin input is not a PDF (missing %PDF header). Pipe PDF bytes with `invo -`, or use --files-from <path> for newline-delimited file paths.");
    }

    Ok(InvoiceInputSpec::stdin(bytes))
}

// Open the file once: extension-check, then read the full contents into a
// buffer, then verify the magic header against the same buffer. Returns the
// buffer if it really is a PDF, None otherwise. There is no second `open()`,
// so symlink swaps or sync-rewrites between checks are impossible.
fn read_pdf_if_valid(path: &Path) -> Option<Vec<u8>> {
    if !has_pdf_extension(path) {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if !is_pdf_bytes(&bytes) {
        return None;
    }
    Some(bytes)
}

fn is_stdin_marker(path: &Path) -> bool {
    path.as_os_str() == "-"
}

fn has_pdf_extension(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.to_string_lossy().to_lowercase() == "pdf")
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
    fn file_list_parsing_ignores_empty_lines_and_comments() {
        let paths = parse_file_list("\n# generated list\n samples/one.pdf \n\nreceipts/two.pdf\n");

        assert_eq!(
            paths,
            vec![
                PathBuf::from("samples/one.pdf"),
                PathBuf::from("receipts/two.pdf")
            ]
        );
    }

    #[test]
    fn expand_inputs_accepts_pdf_paths() {
        let path = PathBuf::from("samples/claude/Receipt-2238-2456-7832.pdf");
        let inputs = expand_inputs(std::slice::from_ref(&path)).expect("sample pdf should expand");

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].path, path.to_string_lossy().to_string());
        assert_eq!(inputs[0].original_name, "Receipt-2238-2456-7832.pdf");
    }

    #[test]
    fn expand_inputs_rejects_duplicate_stdin_markers_without_reading() {
        let error = expand_inputs(&[PathBuf::from("-"), PathBuf::from("-")])
            .expect_err("duplicate stdin markers should fail before reading twice");

        assert!(error.to_string().contains("can only be specified once"));
    }

    #[test]
    fn stdin_input_uses_deterministic_metadata() {
        let input = InvoiceInputSpec::stdin(b"%PDF-1.7\n".to_vec())
            .read()
            .expect("stdin input should materialize");

        assert_eq!(input.path, "stdin.pdf");
        assert_eq!(input.original_name, "stdin.pdf");
    }

    #[test]
    fn path_input_uses_file_name_metadata() {
        let spec = InvoiceInputSpec::from_path_with_bytes(
            Path::new("samples/example.pdf"),
            b"%PDF-1.7\n".to_vec(),
        );

        assert_eq!(spec.path, "samples/example.pdf");
        assert_eq!(spec.original_name, "example.pdf");
    }

    #[test]
    fn expand_inputs_captures_bytes_at_validation_time() {
        // TOCTOU regression: after expand_inputs returns, deleting the source
        // file must not affect what `read()` produces. The bytes are already
        // owned by the InvoiceInputSpec.
        let tmpdir = std::env::temp_dir().join(format!("invo-input-toctou-{}", std::process::id()));
        std::fs::create_dir_all(&tmpdir).expect("tmpdir");
        let pdf_path = tmpdir.join("doc.pdf");
        std::fs::write(&pdf_path, b"%PDF-1.7\nfixture body\n").expect("write pdf");

        let mut specs = expand_inputs(std::slice::from_ref(&pdf_path)).expect("expand");
        assert_eq!(specs.len(), 1);

        // Swap the file out from under us between validation and read.
        std::fs::write(&pdf_path, b"different content").expect("swap pdf");

        let materialized = specs.remove(0).read().expect("read spec");
        assert!(materialized.bytes.starts_with(b"%PDF-1.7"));
        assert!(materialized.bytes.windows(7).any(|w| w == b"fixture"));

        let _ = std::fs::remove_dir_all(&tmpdir);
    }
}
