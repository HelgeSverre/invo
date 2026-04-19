mod config;
mod extraction;
mod llm;
mod pdf;
mod pipeline;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use config::{load_config, Config, OutputFormat, ProviderName};
use extraction::ExtractionResult;
use llm::create_provider;
use pdf::is_pdf;
use pipeline::extract_invoice;

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

    /// Use a specific LLM provider (ollama, openai, anthropic, mistral)
    #[arg(long, value_enum)]
    provider: Option<ProviderName>,

    /// Verbose output (-v for debug, -vv for trace)
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

    if let Some(ref command) = cli.command {
        return handle_command(command, &cli);
    }

    let config = load_config(cli.config.as_ref())?;

    if cli.files.is_empty() {
        eprintln!("No files specified. Use --help for usage information.");
        std::process::exit(1);
    }

    let files = expand_files(&cli.files)?;

    if files.is_empty() {
        eprintln!("No PDF files found matching the specified patterns.");
        std::process::exit(1);
    }

    tracing::info!("Processing {} file(s)", files.len());

    let provider = create_provider(&config, cli.provider)?;

    for file in &files {
        match extract_invoice(file, &config, provider.as_ref()).await {
            Ok(result) => output_result(&result, &config, &cli)?,
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

            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

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

fn output_result(result: &ExtractionResult, config: &Config, cli: &Cli) -> Result<()> {
    let format = cli.format.unwrap_or(config.output.format);

    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(result)?,
        OutputFormat::Yaml => serde_yaml::to_string(result)?,
        OutputFormat::Csv => format!(
            "vendor,date,total,currency,invoice_number\n\"{}\",{},{},{},\"{}\"",
            result.invoice.vendor,
            result.invoice.invoice_date,
            result.invoice.total_amount,
            result.invoice.currency,
            result.invoice.invoice_number.as_deref().unwrap_or("")
        ),
    };

    println!("{}", output);
    Ok(())
}
