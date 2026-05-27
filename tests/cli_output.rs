use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn invo() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_invo"));
    command.current_dir(env!("CARGO_MANIFEST_DIR"));
    command
}

fn temp_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("invo-test-{}-{}", name, std::process::id()));
    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(&path);
    path
}

#[test]
fn verbose_config_validation_keeps_tracing_on_stderr() {
    let output = invo()
        .args(["-v", "-c", "config.yaml", "config", "validate"])
        .env_remove("RUST_LOG")
        .output()
        .expect("invo should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stdout.contains("Status: valid"));
    assert!(stderr.contains("Config {"));
    assert!(!stdout.contains("Config {"));
}

#[test]
fn missing_input_diagnostics_do_not_write_to_stdout() {
    let output = invo()
        .args(["-v", "does-not-exist.pdf"])
        .output()
        .expect("invo should run");

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stdout.is_empty());
    assert!(stderr.contains("File not found"));
    assert!(stderr.contains("No PDF files found"));
}

#[test]
fn stdin_non_pdf_input_returns_actionable_error() {
    let mut child = invo()
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("invo should start");

    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(b"not a pdf")
        .expect("stdin fixture should write");

    let output = child.wait_with_output().expect("invo should finish");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stdout.is_empty());
    assert!(stderr.contains("stdin input is not a PDF"));
    assert!(stderr.contains("--files-from"));
}

#[test]
fn files_from_reads_newline_delimited_paths() {
    let list_path = temp_path("files-from.txt");
    fs::write(&list_path, "README.md\ndoes-not-exist.pdf\n").expect("file list should be written");

    let output = invo()
        .args(["--files-from", list_path.to_str().unwrap(), "-v"])
        .output()
        .expect("invo should run");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stdout.is_empty());
    assert!(stderr.contains("Skipping \"README.md\" - not a PDF file"));
    assert!(stderr.contains("File not found: \"does-not-exist.pdf\""));
    assert!(stderr.contains("No PDF files found"));
}

#[test]
fn config_validate_reports_missing_file_and_effective_defaults() {
    let home = temp_path("missing-home");
    fs::create_dir_all(&home).expect("temp home should be created");

    let output = invo()
        .args(["config", "validate"])
        .env("HOME", &home)
        .output()
        .expect("invo should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");

    assert!(stdout.contains("Status: missing; using built-in defaults"));
    assert!(stdout.contains("Effective defaults are valid."));
}

#[test]
fn config_validate_reports_existing_path_and_accepts_config_after_subcommand() {
    let output = invo()
        .args(["config", "validate", "-c", "config.yaml"])
        .output()
        .expect("invo should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");

    assert!(stdout.contains("Config file: config.yaml"));
    assert!(stdout.contains("Validated: config.yaml"));
    assert!(stdout.contains("Status: valid"));
}

#[test]
fn config_validate_json_is_machine_readable() {
    let output = invo()
        .args(["config", "validate", "-c", "config.yaml", "--json"])
        .output()
        .expect("invo should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value = serde_json::from_str(&stdout).expect("validation output should be JSON");

    assert_eq!(parsed["status"], "valid");
    assert_eq!(parsed["exists"], true);
    assert_eq!(parsed["strict"], true);
}

#[test]
fn config_validate_rejects_unknown_key_with_path() {
    let config_path = temp_path("unknown-key.yaml");
    fs::write(&config_path, "identity:\n  vatnumbber: NO123\n")
        .expect("config fixture should be written");

    let output = invo()
        .args(["-c", config_path.to_str().unwrap(), "config", "validate"])
        .output()
        .expect("invo should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stderr.contains("vatnumbber"));
}

#[test]
fn config_subcommands_are_not_silent() {
    let output = invo().arg("config").output().expect("invo should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stderr.contains("Usage:"));
    assert!(stderr.contains("validate"));
}

#[test]
fn init_respects_explicit_config_path_and_requires_force_to_overwrite() {
    let config_path = temp_path("custom-config.yaml");

    let output = invo()
        .args(["init", "-c", config_path.to_str().unwrap()])
        .output()
        .expect("invo should run");

    assert!(output.status.success());
    assert!(config_path.exists());

    // init success message must land on stderr so stdout stays data-only.
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stdout.is_empty(), "stdout should be empty, got: {stdout}");
    assert!(
        stderr.contains("Created config file at"),
        "stderr should report creation, got: {stderr}"
    );

    let second = invo()
        .args(["init", "-c", config_path.to_str().unwrap()])
        .output()
        .expect("invo should run");

    assert!(!second.status.success());
    let stderr = String::from_utf8(second.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--force"));

    let forced = invo()
        .args(["init", "-c", config_path.to_str().unwrap(), "--force"])
        .output()
        .expect("invo should run");

    assert!(forced.status.success());
}

#[test]
fn help_includes_copy_pasteable_examples() {
    let top = invo().arg("--help").output().expect("invo should run");
    assert!(top.status.success());
    let top_stdout = String::from_utf8(top.stdout).expect("stdout should be utf-8");
    assert!(top_stdout.contains("Examples:"));
    assert!(top_stdout.contains("invo \"receipts/**/*.pdf\" --json"));

    let config = invo()
        .args(["config", "--help"])
        .output()
        .expect("invo should run");
    assert!(config.status.success());
    let config_stdout = String::from_utf8(config.stdout).expect("stdout should be utf-8");
    assert!(config_stdout.contains("invo config show --effective --json"));
}

#[test]
fn agent_context_json_is_available() {
    let output = invo()
        .args(["agent-context", "--json", "-c", "config.yaml"])
        .output()
        .expect("invo should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value = serde_json::from_str(&stdout).expect("agent context should be JSON");

    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(
        parsed["commands"]["extract"]["stdout_contract"],
        "data only"
    );
    assert!(parsed["config"]["active_keys"]
        .as_array()
        .expect("active keys should be array")
        .iter()
        .any(|value| value == "identity.vat_number"));
}
