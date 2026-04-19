mod config;
mod extraction;
mod llm;
mod pdf;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use config::{load_config, Config, OutputFormat};
use extraction::{
    extraction_json_schema, extraction_system_prompt, extraction_user_message, ExtractionMeta,
    ExtractionResult, FileInfo, Invoice,
};
use llm::{create_provider, LlmRequest};
use pdf::{extract_text, is_pdf};

/// Debug log file path
const DEBUG_LOG_PATH: &str = "/Users/helge/code/invo/debug.log";

/// Write a debug message to the log file
fn debug_log(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(DEBUG_LOG_PATH)
    {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }
}

#[derive(Parser)]
#[command(name = "invo")]
#[command(about = "Extract structured data from PDF receipts/invoices using LLMs")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// PDF file(s) to process (supports glob patterns)
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Output format (json, yaml, csv)
    #[arg(short, long, value_enum)]
    format: Option<OutputFormat>,

    /// Config file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Force OCR even if native text extraction succeeds
    #[arg(long)]
    force_ocr: bool,

    /// Output to stdout only (no file actions)
    #[arg(long)]
    stdout: bool,

    /// Dry run - show what would happen without making changes
    #[arg(long)]
    dry_run: bool,

    /// Use a specific LLM provider
    #[arg(long)]
    provider: Option<String>,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new config file
    Init,

    /// Validate the config file
    Config {
        #[arg(long)]
        validate: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    // Handle subcommands
    if let Some(ref command) = cli.command {
        return handle_command(command, &cli);
    }

    // Load config
    let config = load_config(cli.config.as_ref())?;

    // Check for files to process
    if cli.files.is_empty() {
        eprintln!("No files specified. Use --help for usage information.");
        std::process::exit(1);
    }

    // Expand glob patterns and collect files
    let files = expand_files(&cli.files)?;

    if files.is_empty() {
        eprintln!("No PDF files found matching the specified patterns.");
        std::process::exit(1);
    }

    tracing::info!("Processing {} file(s)", files.len());

    // Process files
    for file in &files {
        match process_file(file, &config, &cli).await {
            Ok(result) => {
                output_result(&result, &config, &cli)?;
            }
            Err(e) => {
                tracing::error!("Failed to process {:?}: {}", file, e);
                if files.len() == 1 {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

fn handle_command(command: &Commands, cli: &Cli) -> Result<()> {
    match command {
        Commands::Init => {
            let config_path = config::default_config_path();
            if config_path.exists() {
                eprintln!("Config file already exists at {:?}", config_path);
                std::process::exit(1);
            }

            // Create parent directories
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Write default config
            let default_config = include_str!("../default_config.yaml");
            std::fs::write(&config_path, default_config)?;

            println!("Created config file at {:?}", config_path);
            Ok(())
        }
        Commands::Config { validate } => {
            if *validate {
                let config = load_config(cli.config.as_ref())?;
                println!("Config is valid!");
                tracing::debug!("{:#?}", config);
            }
            Ok(())
        }
    }
}

fn expand_files(patterns: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        let pattern_str = pattern.to_string_lossy();

        // Check if it's a glob pattern
        if pattern_str.contains('*') || pattern_str.contains('?') {
            for entry in glob::glob(&pattern_str)? {
                let path = entry?;
                if is_pdf(&path) {
                    files.push(path);
                }
            }
        } else if pattern.exists() && is_pdf(pattern) {
            files.push(pattern.clone());
        } else if pattern.exists() {
            tracing::warn!("Skipping {:?} - not a PDF file", pattern);
        } else {
            tracing::warn!("File not found: {:?}", pattern);
        }
    }

    Ok(files)
}

async fn process_file(path: &PathBuf, config: &Config, _cli: &Cli) -> Result<ExtractionResult> {
    tracing::info!("Processing: {:?}", path);
    let start = Instant::now();

    // Extract text from PDF
    let pdf_result = extract_text(path).context("Failed to extract text from PDF")?;

    if pdf_result.text.is_empty() {
        anyhow::bail!("No text extracted from PDF");
    }

    tracing::debug!(
        "Extracted {} chars from {} pages",
        pdf_result.text.len(),
        pdf_result.page_count
    );

    // Create LLM provider
    let provider = create_provider(config)?;

    // Build extraction request
    let system_prompt = extraction_system_prompt(&config.identity, &config.vendors);
    let user_message = extraction_user_message(&pdf_result.text);
    let json_schema = extraction_json_schema();

    let request = LlmRequest {
        system_prompt,
        user_message,
        json_schema: Some(json_schema),
        temperature: config.llm.extraction.temperature,
        max_tokens: config.llm.extraction.max_tokens,
    };

    // Send to LLM
    tracing::debug!("Sending to LLM provider: {}", provider.name());
    debug_log(&format!("=== LLM Request for {:?} ===", path));
    debug_log(&format!("Provider: {}", provider.name()));
    debug_log(&format!("System prompt:\n{}", request.system_prompt));
    debug_log(&format!("Text length: {} chars", request.user_message.len()));

    let response = provider.complete(request).await?;

    debug_log(&format!("=== LLM Response ===\n{}", response.content));

    // Parse response
    let invoice: Invoice = match serde_json::from_str(&response.content) {
        Ok(inv) => inv,
        Err(e) => {
            debug_log(&format!("=== PARSE ERROR ===\n{}\n\nResponse was:\n{}", e, response.content));
            return Err(anyhow::anyhow!("Failed to parse LLM response: {}", e))
                .with_context(|| format!("Raw response:\n{}", response.content));
        }
    };

    let processing_time = start.elapsed();

    // Build result
    let confidence = invoice.confidence.unwrap_or(0.9);
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
            confidence,
            processing_time_ms: processing_time.as_millis() as u64,
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

fn output_result(result: &ExtractionResult, config: &Config, cli: &Cli) -> Result<()> {
    let format = cli.format.unwrap_or(config.output.format);

    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(result)?,
        OutputFormat::Yaml => serde_yaml::to_string(result)?,
        OutputFormat::Csv => {
            // Simple CSV output for single invoice
            format!(
                "vendor,date,total,currency,invoice_number\n\"{}\",{},{},{},\"{}\"",
                result.invoice.vendor,
                result.invoice.invoice_date,
                result.invoice.total_amount,
                result.invoice.currency,
                result.invoice.invoice_number.as_deref().unwrap_or("")
            )
        }
    };

    println!("{}", output);
    Ok(())
}
