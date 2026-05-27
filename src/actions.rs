use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail, Context, Result};

use crate::config::{
    Action, ExportAction, FileAction, FileActionKind, HookAction, OnError, OutputFormat,
};
use crate::invoice::ExtractionResult;

/// Shared across parallel batch workers (and trivially shared in single-file
/// mode too) so two workers that resolve to the same filesystem destination
/// don't clobber each other. Decision-4 documents the "first writer wins,
/// second logs WARN and skips" policy.
///
/// Caller responsibility: build one registry per batch invocation and clone
/// the Arc into each worker's closure (or, in single-file mode, into the
/// one and only call to `run_actions`). Hook actions don't participate
/// because their destinations are user-script-defined and invisible here.
pub type DestinationRegistry = Arc<Mutex<HashSet<PathBuf>>>;

/// Build a fresh registry. Call once per batch.
pub fn new_destination_registry() -> DestinationRegistry {
    Arc::new(Mutex::new(HashSet::new()))
}

/// Try to claim a destination path for writing. Returns true when the caller
/// is the first to claim it (and may proceed); false when another worker has
/// already claimed it (caller should log WARN and skip).
fn try_claim_destination(registry: &DestinationRegistry, dest: &Path) -> bool {
    let mut set = registry
        .lock()
        .expect("destination registry mutex was poisoned by a panicking worker");
    set.insert(dest.to_path_buf())
}

/// Behavior shared by every post-extraction action variant. Each variant owns
/// its own label, enabled-check, and execution — the dispatch in `run_actions`
/// stays generic over the trait. `impl PostAction for Action` delegates to the
/// concrete inner type so callers can keep iterating over `&[Action]` from
/// config without caring which variant is which.
trait PostAction {
    fn label(&self) -> &'static str;
    fn enabled(&self) -> bool;
    fn run(
        &self,
        result: &ExtractionResult,
        source_pdf_bytes: &[u8],
        placeholders: &HashMap<&'static str, String>,
        registry: &DestinationRegistry,
    ) -> Result<()>;
}

impl PostAction for FileAction {
    fn label(&self) -> &'static str {
        match self.kind {
            FileActionKind::Copy => "copy",
            FileActionKind::Move => "move",
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn run(
        &self,
        result: &ExtractionResult,
        source_pdf_bytes: &[u8],
        placeholders: &HashMap<&'static str, String>,
        registry: &DestinationRegistry,
    ) -> Result<()> {
        match self.kind {
            FileActionKind::Copy => {
                run_copy(self, result, source_pdf_bytes, placeholders, registry)
            }
            FileActionKind::Move => {
                run_move(self, result, source_pdf_bytes, placeholders, registry)
            }
        }
    }
}

impl PostAction for ExportAction {
    fn label(&self) -> &'static str {
        "export"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn run(
        &self,
        result: &ExtractionResult,
        _source_pdf_bytes: &[u8],
        placeholders: &HashMap<&'static str, String>,
        registry: &DestinationRegistry,
    ) -> Result<()> {
        run_export(self, result, placeholders, registry)
    }
}

impl PostAction for HookAction {
    fn label(&self) -> &'static str {
        "hook"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn run(
        &self,
        result: &ExtractionResult,
        _source_pdf_bytes: &[u8],
        placeholders: &HashMap<&'static str, String>,
        _registry: &DestinationRegistry,
    ) -> Result<()> {
        run_hook(self, result, placeholders)
    }
}

impl PostAction for Action {
    fn label(&self) -> &'static str {
        match self {
            Action::File(a) => a.label(),
            Action::Export(a) => a.label(),
            Action::Hook(a) => a.label(),
        }
    }

    fn enabled(&self) -> bool {
        match self {
            Action::File(a) => a.enabled(),
            Action::Export(a) => a.enabled(),
            Action::Hook(a) => a.enabled(),
        }
    }

    fn run(
        &self,
        result: &ExtractionResult,
        source_pdf_bytes: &[u8],
        placeholders: &HashMap<&'static str, String>,
        registry: &DestinationRegistry,
    ) -> Result<()> {
        match self {
            Action::File(a) => a.run(result, source_pdf_bytes, placeholders, registry),
            Action::Export(a) => a.run(result, source_pdf_bytes, placeholders, registry),
            Action::Hook(a) => a.run(result, source_pdf_bytes, placeholders, registry),
        }
    }
}

// Run all configured post-processing actions for one successful extraction.
// Disabled actions are skipped silently. Per-action failures log and continue
// unless the action is a Hook with on_error: Abort, which propagates the error
// up so the caller can decide what to do with the batch.
//
// The destination registry is shared across parallel workers; see decision-4
// for the "first writer wins" policy. Single-file callers can pass a fresh
// registry — there's nothing to collide with.
pub fn run_actions(
    actions: &[Action],
    result: &ExtractionResult,
    source_pdf_bytes: &[u8],
    registry: &DestinationRegistry,
) -> Result<()> {
    let placeholders = build_placeholders(result);
    for action in actions {
        if !action.enabled() {
            continue;
        }
        if let Err(err) = action.run(result, source_pdf_bytes, &placeholders, registry) {
            match action {
                Action::Hook(h) if matches!(h.on_error, OnError::Abort) => return Err(err),
                _ => tracing::error!("Action failed (continuing): {:?}: {err:#}", action.label()),
            }
        }
    }
    Ok(())
}

fn run_copy(
    action: &FileAction,
    result: &ExtractionResult,
    source_pdf_bytes: &[u8],
    placeholders: &HashMap<&'static str, String>,
    registry: &DestinationRegistry,
) -> Result<()> {
    let dest = resolve_file_dest(action, result, placeholders)?;
    if !try_claim_destination(registry, &dest) {
        tracing::warn!(
            "copy action skipped: destination {} already claimed by another worker (source: {})",
            dest.display(),
            result.file.path
        );
        return Ok(());
    }
    ensure_parent(&dest, action.create_dirs)?;
    write_pdf_to(&dest, &result.file.path, source_pdf_bytes)
        .with_context(|| format!("copy action -> {}", dest.display()))?;
    tracing::info!("Action: copied {} -> {}", result.file.path, dest.display());
    Ok(())
}

fn run_move(
    action: &FileAction,
    result: &ExtractionResult,
    source_pdf_bytes: &[u8],
    placeholders: &HashMap<&'static str, String>,
    registry: &DestinationRegistry,
) -> Result<()> {
    let dest = resolve_file_dest(action, result, placeholders)?;
    if !try_claim_destination(registry, &dest) {
        // Decision-4: skip the move AND do NOT delete the source. Another
        // worker wrote a different file's contents at this destination, so
        // deleting the source here would lose user data.
        tracing::warn!(
            "move action skipped: destination {} already claimed by another worker (source preserved: {})",
            dest.display(),
            result.file.path
        );
        return Ok(());
    }
    ensure_parent(&dest, action.create_dirs)?;
    write_pdf_to(&dest, &result.file.path, source_pdf_bytes)
        .with_context(|| format!("move action -> {}", dest.display()))?;
    // Remove the source AFTER the copy has succeeded. If source was stdin,
    // there's nothing to remove.
    let source = Path::new(&result.file.path);
    if source.exists() && source != dest {
        if let Err(err) = fs::remove_file(source) {
            tracing::warn!(
                "move action wrote {} but failed to remove source {}: {err}",
                dest.display(),
                source.display()
            );
        }
    }
    tracing::info!("Action: moved {} -> {}", result.file.path, dest.display());
    Ok(())
}

fn run_export(
    action: &ExportAction,
    result: &ExtractionResult,
    placeholders: &HashMap<&'static str, String>,
    registry: &DestinationRegistry,
) -> Result<()> {
    let dest_dir = expand(&action.destination, placeholders);
    let filename = match &action.filename {
        Some(t) => expand(t, placeholders),
        None => default_export_filename(result, action.format),
    };
    let dest = PathBuf::from(dest_dir).join(sanitize_filename(&filename));
    if !try_claim_destination(registry, &dest) {
        tracing::warn!(
            "export action skipped: destination {} already claimed by another worker (source: {})",
            dest.display(),
            result.file.path
        );
        return Ok(());
    }
    ensure_parent(&dest, true)?;

    let serialized = match action.format {
        OutputFormat::Json => serde_json::to_string_pretty(result)?,
        OutputFormat::Yaml => serde_yaml::to_string(result)?,
        OutputFormat::Csv => render_single_csv(result),
    };
    fs::write(&dest, serialized).with_context(|| format!("export -> {}", dest.display()))?;
    tracing::info!(
        "Action: exported {} -> {}",
        result.file.path,
        dest.display()
    );
    Ok(())
}

fn run_hook(
    action: &HookAction,
    result: &ExtractionResult,
    placeholders: &HashMap<&'static str, String>,
) -> Result<()> {
    let expanded_args: Vec<String> = action
        .args
        .iter()
        .map(|a| expand(a, placeholders))
        .collect();

    let output = Command::new(&action.command)
        .args(&expanded_args)
        .env("INVO_VENDOR", placeholder_or_empty(placeholders, "vendor"))
        .env("INVO_TOTAL", placeholder_or_empty(placeholders, "total"))
        .env(
            "INVO_CURRENCY",
            placeholder_or_empty(placeholders, "currency"),
        )
        .env("INVO_DATE", placeholder_or_empty(placeholders, "date"))
        .env(
            "INVO_INVOICE_NUMBER",
            placeholder_or_empty(placeholders, "invoice_number"),
        )
        .env("INVO_PATH", &result.file.path)
        .env("INVO_HASH", &result.file.hash)
        .env("INVO_ORIGINAL_NAME", &result.file.original_name)
        .output()
        .with_context(|| format!("hook: failed to spawn {}", action.command))?;

    if !output.stdout.is_empty() {
        tracing::debug!(
            "hook {} stdout: {}",
            action.command,
            String::from_utf8_lossy(&output.stdout).trim_end()
        );
    }
    if !output.stderr.is_empty() {
        tracing::debug!(
            "hook {} stderr: {}",
            action.command,
            String::from_utf8_lossy(&output.stderr).trim_end()
        );
    }

    if !output.status.success() {
        return Err(anyhow!(
            "hook `{}` exited with {} (args: {:?})",
            action.command,
            output.status,
            expanded_args
        ));
    }
    tracing::info!("Action: hook {} ok", action.command);
    Ok(())
}

// --- placeholders -------------------------------------------------------

fn build_placeholders(result: &ExtractionResult) -> HashMap<&'static str, String> {
    let mut map = HashMap::new();
    let date = result.invoice.invoice_date;
    map.insert("vendor", result.invoice.vendor.clone());
    map.insert("date", date.format("%Y-%m-%d").to_string());
    map.insert("year", date.format("%Y").to_string());
    map.insert("month", date.format("%m").to_string());
    map.insert("day", date.format("%d").to_string());
    map.insert(
        "invoice_number",
        result.invoice.invoice_number.clone().unwrap_or_default(),
    );
    map.insert("currency", result.invoice.currency.clone());
    map.insert("total", format!("{}", result.invoice.total_amount));
    map.insert("hash", result.file.hash.clone());
    map.insert("original_name", result.file.original_name.clone());
    let stem = Path::new(&result.file.original_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| result.file.original_name.clone());
    map.insert("stem", stem);
    map
}

fn placeholder_or_empty(map: &HashMap<&'static str, String>, key: &str) -> String {
    map.get(key).cloned().unwrap_or_default()
}

// Naive `{name}` substitution. We intentionally don't pull in handlebars/tera;
// the placeholder set is small and known.
fn expand(template: &str, placeholders: &HashMap<&'static str, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in placeholders {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

// --- helpers ------------------------------------------------------------

fn resolve_file_dest(
    action: &FileAction,
    result: &ExtractionResult,
    placeholders: &HashMap<&'static str, String>,
) -> Result<PathBuf> {
    let dest_dir = expand(&action.destination, placeholders);
    let filename = match &action.filename {
        Some(t) => expand(t, placeholders),
        None => result.file.original_name.clone(),
    };
    Ok(PathBuf::from(dest_dir).join(sanitize_filename(&filename)))
}

fn ensure_parent(dest: &Path, create_dirs: bool) -> Result<()> {
    if let Some(parent) = dest.parent() {
        if parent.as_os_str().is_empty() {
            return Ok(());
        }
        if create_dirs {
            fs::create_dir_all(parent)
                .with_context(|| format!("create destination dir {}", parent.display()))?;
        } else if !parent.exists() {
            bail!(
                "destination directory does not exist: {} (set create_dirs: true on the action)",
                parent.display()
            );
        }
    }
    Ok(())
}

fn write_pdf_to(dest: &Path, source_path: &str, source_bytes: &[u8]) -> Result<()> {
    // Prefer fs::copy when the source is a real file (preserves permissions
    // on platforms where that matters). Fall back to writing the captured
    // bytes for stdin or when the source disappeared mid-batch.
    let source = Path::new(source_path);
    if source.exists() {
        fs::copy(source, dest).map(|_| ())?;
    } else {
        fs::write(dest, source_bytes)?;
    }
    Ok(())
}

fn default_export_filename(result: &ExtractionResult, format: OutputFormat) -> String {
    let stem = Path::new(&result.file.original_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "invoice".to_string());
    let ext = match format {
        OutputFormat::Json => "json",
        OutputFormat::Yaml => "yaml",
        OutputFormat::Csv => "csv",
    };
    format!("{stem}.{ext}")
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

fn render_single_csv(result: &ExtractionResult) -> String {
    // Same column set as the batch CSV renderer in src/output.rs (kept in
    // sync deliberately so export action output is interchangeable).
    let header = "vendor,date,total,currency,invoice_number";
    let row = format!(
        "\"{}\",{},{},{},\"{}\"",
        result.invoice.vendor.replace('"', "\"\""),
        result.invoice.invoice_date,
        result.invoice.total_amount,
        result.invoice.currency,
        result
            .invoice
            .invoice_number
            .as_deref()
            .unwrap_or("")
            .replace('"', "\"\"")
    );
    format!("{header}\n{row}\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ExportAction, FileAction, HookAction, OnError};
    use crate::invoice::{ExtractionMeta, FileInfo, Invoice};
    use chrono::NaiveDate;

    fn fixture(vendor: &str, original_name: &str, source_path: &str) -> ExtractionResult {
        ExtractionResult {
            file: FileInfo {
                path: source_path.to_string(),
                original_name: original_name.to_string(),
                hash: "abc1234567890def".to_string(),
                processed_at: "2026-05-27T00:00:00Z".to_string(),
            },
            invoice: Invoice {
                vendor: vendor.to_string(),
                invoice_date: NaiveDate::from_ymd_opt(2026, 5, 27).unwrap(),
                total_amount: 42.5,
                currency: "USD".to_string(),
                invoice_number: Some("INV-001".to_string()),
                due_date: None,
                subtotal: None,
                tax_amount: None,
                tax_rate: None,
                discount_amount: None,
                vendor_website: None,
                vendor_email: None,
                vendor_address: None,
                payment_method: None,
                card_last_four: None,
                items: vec![],
                category: None,
                description: None,
                buyer_name: None,
                confidence: None,
            },
            extraction: ExtractionMeta {
                provider: "ollama".into(),
                model: "qwen2.5:1.5b".into(),
                ocr_used: false,
                processing_time_ms: 1,
            },
            identity_match: None,
        }
    }

    fn tmpdir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("invo-actions-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).expect("tmpdir");
        p
    }

    #[test]
    fn expand_substitutes_all_placeholders() {
        let result = fixture("Acme", "receipt.pdf", "/tmp/receipt.pdf");
        let placeholders = build_placeholders(&result);
        let expanded = expand(
            "{year}/{month}/{vendor}-{invoice_number}-{total}-{currency}.{stem}",
            &placeholders,
        );
        assert_eq!(expanded, "2026/05/Acme-INV-001-42.5-USD.receipt");
    }

    #[test]
    fn sanitize_filename_replaces_path_separators_and_control_chars() {
        assert_eq!(sanitize_filename("Acme/Inc.pdf"), "Acme_Inc.pdf");
        assert_eq!(sanitize_filename("file:name\\x.pdf"), "file_name_x.pdf");
        assert_eq!(sanitize_filename("a\0b"), "a_b");
    }

    #[test]
    fn copy_action_writes_file_to_destination() {
        let dir = tmpdir("copy");
        let src = dir.join("source.pdf");
        fs::write(&src, b"%PDF-1.7\nbody\n").unwrap();
        let result = fixture("Acme", "receipt.pdf", src.to_str().unwrap());
        let action = Action::File(FileAction {
            kind: FileActionKind::Copy,
            enabled: true,
            destination: dir.join("out/{year}").to_string_lossy().to_string(),
            filename: Some("{vendor}-{invoice_number}.pdf".into()),
            create_dirs: true,
        });
        run_actions(
            &[action],
            &result,
            b"%PDF-1.7\nbody\n",
            &new_destination_registry(),
        )
        .expect("copy ok");
        assert!(dir.join("out/2026/Acme-INV-001.pdf").exists());
        // Source still there for Copy.
        assert!(src.exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn move_action_removes_source() {
        let dir = tmpdir("move");
        let src = dir.join("source.pdf");
        fs::write(&src, b"%PDF-1.7\n").unwrap();
        let result = fixture("Acme", "receipt.pdf", src.to_str().unwrap());
        let action = Action::File(FileAction {
            kind: FileActionKind::Move,
            enabled: true,
            destination: dir.join("out").to_string_lossy().to_string(),
            filename: None,
            create_dirs: true,
        });
        run_actions(
            &[action],
            &result,
            b"%PDF-1.7\n",
            &new_destination_registry(),
        )
        .expect("move ok");
        assert!(dir.join("out/receipt.pdf").exists());
        assert!(!src.exists(), "source should be removed after Move");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn export_action_writes_format_to_destination() {
        let dir = tmpdir("export");
        let result = fixture("Acme", "receipt.pdf", "/nowhere");
        let action = Action::Export(ExportAction {
            enabled: true,
            format: OutputFormat::Json,
            destination: dir.to_string_lossy().to_string(),
            filename: Some("{vendor}-{date}.json".into()),
        });
        run_actions(&[action], &result, b"", &new_destination_registry()).expect("export ok");
        let body = fs::read_to_string(dir.join("Acme-2026-05-27.json")).unwrap();
        assert!(body.contains("\"vendor\": \"Acme\""));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn export_action_defaults_filename_to_stem_with_format_ext() {
        let dir = tmpdir("export-default");
        let result = fixture("Acme", "receipt-A.pdf", "/nowhere");
        let action = Action::Export(ExportAction {
            enabled: true,
            format: OutputFormat::Csv,
            destination: dir.to_string_lossy().to_string(),
            filename: None,
        });
        run_actions(&[action], &result, b"", &new_destination_registry()).expect("export ok");
        let body = fs::read_to_string(dir.join("receipt-A.csv")).unwrap();
        assert!(body.starts_with("vendor,date,total,currency,invoice_number\n"));
        assert!(body.contains("Acme"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn disabled_actions_are_skipped() {
        let dir = tmpdir("disabled");
        let result = fixture("Acme", "receipt.pdf", "/nowhere");
        let action = Action::Export(ExportAction {
            enabled: false,
            format: OutputFormat::Json,
            destination: dir.to_string_lossy().to_string(),
            filename: Some("should-not-appear.json".into()),
        });
        run_actions(&[action], &result, b"", &new_destination_registry()).expect("ok");
        assert!(!dir.join("should-not-appear.json").exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hook_action_runs_command_and_passes_env() {
        let dir = tmpdir("hook");
        let marker = dir.join("marker.txt");
        let action = Action::Hook(HookAction {
            enabled: true,
            command: "sh".into(),
            args: vec![
                "-c".into(),
                format!(
                    "echo \"$INVO_VENDOR:$INVO_TOTAL:$INVO_CURRENCY:$INVO_DATE:{}\" > {}",
                    "{stem}",
                    marker.to_string_lossy()
                ),
            ],
            on_error: OnError::Continue,
        });
        let result = fixture("Acme", "receipt.pdf", "/nowhere");
        run_actions(&[action], &result, b"", &new_destination_registry()).expect("hook ok");
        let body = fs::read_to_string(&marker).unwrap();
        assert!(
            body.contains("Acme:42.5:USD:2026-05-27:receipt"),
            "got {body:?}"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hook_action_on_error_continue_swallows_failure() {
        let dir = tmpdir("hook-cont");
        let succeed = dir.join("succeed.txt");
        let actions = vec![
            Action::Hook(HookAction {
                enabled: true,
                command: "sh".into(),
                args: vec!["-c".into(), "exit 3".into()],
                on_error: OnError::Continue,
            }),
            Action::Hook(HookAction {
                enabled: true,
                command: "sh".into(),
                args: vec!["-c".into(), format!("touch {}", succeed.to_string_lossy())],
                on_error: OnError::Continue,
            }),
        ];
        let result = fixture("Acme", "r.pdf", "/nowhere");
        run_actions(&actions, &result, b"", &new_destination_registry())
            .expect("continue should not propagate");
        assert!(succeed.exists(), "second hook should still have run");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn copy_action_with_colliding_destination_is_skipped_and_first_wins() {
        let dir = tmpdir("collide-copy");
        let src_a = dir.join("a.pdf");
        let src_b = dir.join("b.pdf");
        fs::write(&src_a, b"FIRST").unwrap();
        fs::write(&src_b, b"SECOND").unwrap();
        let dest_template = dir.join("out").to_string_lossy().to_string();
        let registry = new_destination_registry();

        // Two file actions resolving to the same destination filename. The
        // first worker claims the path; the second must skip without writing.
        let action_a = Action::File(FileAction {
            kind: FileActionKind::Copy,
            enabled: true,
            destination: dest_template.clone(),
            filename: Some("same.pdf".into()),
            create_dirs: true,
        });
        let action_b = Action::File(FileAction {
            kind: FileActionKind::Copy,
            enabled: true,
            destination: dest_template,
            filename: Some("same.pdf".into()),
            create_dirs: true,
        });

        let result_a = fixture("Acme", "a.pdf", src_a.to_str().unwrap());
        let result_b = fixture("Beta", "b.pdf", src_b.to_str().unwrap());
        run_actions(&[action_a], &result_a, b"FIRST", &registry).expect("first copy ok");
        run_actions(&[action_b], &result_b, b"SECOND", &registry)
            .expect("second copy must be Ok (skipped, not errored)");

        // Destination must hold the first writer's bytes — second was skipped.
        let body = fs::read(dir.join("out/same.pdf")).expect("dest exists");
        assert_eq!(body, b"FIRST", "first writer must win the collision");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn move_action_with_colliding_destination_skips_and_preserves_source() {
        let dir = tmpdir("collide-move");
        let src_a = dir.join("a.pdf");
        let src_b = dir.join("b.pdf");
        fs::write(&src_a, b"FIRST").unwrap();
        fs::write(&src_b, b"SECOND").unwrap();
        let dest_template = dir.join("out").to_string_lossy().to_string();
        let registry = new_destination_registry();

        let action_a = Action::File(FileAction {
            kind: FileActionKind::Move,
            enabled: true,
            destination: dest_template.clone(),
            filename: Some("same.pdf".into()),
            create_dirs: true,
        });
        let action_b = Action::File(FileAction {
            kind: FileActionKind::Move,
            enabled: true,
            destination: dest_template,
            filename: Some("same.pdf".into()),
            create_dirs: true,
        });

        let result_a = fixture("Acme", "a.pdf", src_a.to_str().unwrap());
        let result_b = fixture("Beta", "b.pdf", src_b.to_str().unwrap());
        run_actions(&[action_a], &result_a, b"FIRST", &registry).expect("first move ok");
        run_actions(&[action_b], &result_b, b"SECOND", &registry)
            .expect("second move must be Ok (skipped)");

        // First source is gone (moved); second source must still exist
        // because the move was skipped on collision.
        assert!(!src_a.exists(), "first source must be removed by Move");
        assert!(
            src_b.exists(),
            "second source must be preserved when Move is skipped on collision (decision-4)"
        );
        let body = fs::read(dir.join("out/same.pdf")).unwrap();
        assert_eq!(body, b"FIRST");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hook_action_on_error_abort_propagates() {
        let action = Action::Hook(HookAction {
            enabled: true,
            command: "sh".into(),
            args: vec!["-c".into(), "exit 3".into()],
            on_error: OnError::Abort,
        });
        let result = fixture("Acme", "r.pdf", "/nowhere");
        let err = run_actions(&[action], &result, b"", &new_destination_registry())
            .expect_err("abort should fail");
        assert!(err.to_string().contains("exited"));
    }
}
