mod actions;
mod agent_context;
mod config;
mod config_commands;
mod extractor;
mod input;
mod invoice;
mod llm;
mod output;
mod pdf;
mod prompt;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::path::PathBuf;

use agent_context::{handle_agent_context, AgentContextOptions};
use config::{load_config, OutputFormat, ProviderName};
use config_commands::{handle_config_command, handle_init, ConfigCommandOptions};
use extractor::extract_invoice;
use input::{collect_inputs, InputOptions};
use llm::LlmContext;
use output::{
    is_ndjson_stream, render_results, validate_output_flags, write_ndjson_record, OutputOptions,
};

#[derive(Parser)]
#[command(name = "invo")]
#[command(about = "Extract structured data from PDF receipts/invoices using LLMs")]
#[command(version)]
#[command(
    after_help = "Examples:\n  invo receipt.pdf\n  invo - < receipt.pdf\n  invo --files-from receipts.txt --json\n  invo \"receipts/**/*.pdf\" --json > receipts.ndjson\n  invo \"receipts/**/*.pdf\" --json-array > receipts.json\n  invo -c config.yaml config validate\n  invo config show --effective --json\n  invo agent-context --json"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// PDF file(s) to process (supports glob patterns). Use '-' to read PDF bytes from stdin.
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Read newline-delimited file paths or globs from a file. Use '-' to read the list from stdin.
    #[arg(long, value_name = "PATH")]
    files_from: Option<PathBuf>,

    /// Output format (json, yaml, csv)
    #[arg(short, long, value_enum)]
    format: Option<OutputFormat>,

    /// Emit machine-readable JSON where supported. For extraction, compact JSON; batches are NDJSON.
    #[arg(long, global = true)]
    json: bool,

    /// Emit one JSON array containing all successful results.
    #[arg(long)]
    json_array: bool,

    /// Pretty-print JSON for single-result human display.
    #[arg(long)]
    pretty: bool,

    /// Config file path
    #[arg(short, long, value_name = "FILE", global = true)]
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
    #[command(
        after_help = "Examples:\n  invo init\n  invo -c ./config.yaml init\n  invo init -c ./config.yaml --force"
    )]
    Init {
        /// Overwrite an existing config file
        #[arg(long)]
        force: bool,
    },

    /// Inspect, validate, and diagnose config files
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  invo config path\n  invo -c config.yaml config validate\n  invo config validate -c config.yaml --json\n  invo config show --effective --json\n  invo config doctor\n  invo config schema --json"
    )]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Emit machine-readable CLI/config context for agents
    #[command(
        name = "agent-context",
        after_help = "Examples:\n  invo agent-context --json"
    )]
    AgentContext,

    /// Generate shell completions
    #[command(
        after_help = "Examples:\n  invo completions --shell zsh > ~/.zsh/completions/_invo\n  invo completions --shell bash > ~/.local/share/bash-completion/completions/invo\n  invo completions --shell fish > ~/.config/fish/completions/invo.fish\n\n  See README for shell-specific installation instructions."
    )]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum, long)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub(crate) enum ConfigCommand {
    /// Print the config path invo will use
    Path,

    /// Show the config file or effective config
    Show {
        /// Show the effective config after defaults are applied
        #[arg(long)]
        effective: bool,
    },

    /// Validate the config file, or report that built-in defaults are in use
    Validate {
        /// Kept for readability; validation is strict by default
        #[arg(long)]
        strict: bool,
    },

    /// Diagnose parsed-only or inactive config keys
    Doctor,

    /// Emit the JSON schema for config.yaml
    Schema,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    init_logging(cli.verbose);

    if let Some(ref command) = cli.command {
        return handle_command(command, &cli);
    }

    let config = load_config(cli.config.as_ref())?;
    let output_options = output_options(&cli);
    validate_output_flags(&output_options)?;

    let input_options = input_options(&cli);
    let inputs = collect_inputs(&input_options)?;

    if inputs.is_empty() {
        if cli.files.is_empty() && cli.files_from.is_none() {
            eprintln!("No files specified. Use --help for usage information.");
        } else {
            eprintln!("No PDF files found matching the specified patterns.");
        }
        std::process::exit(1);
    }

    tracing::info!("Processing {} file(s)", inputs.len());

    // Build the LLM provider client ONCE and reuse across the batch so
    // reqwest's connection pool survives between extractions.
    let llm = LlmContext::build(&config, cli.provider)?;

    let input_count = inputs.len();
    let streaming = is_ndjson_stream(&config, &output_options, input_count);
    let mut results = Vec::new();
    let mut success_count: usize = 0;

    for input_spec in inputs {
        let input_path = input_spec.path.clone();
        let input = match input_spec.read() {
            Ok(input) => input,
            Err(error) => {
                tracing::error!("Failed to process {}: {}", input_path, error);
                if input_count == 1 {
                    return Err(error);
                }
                continue;
            }
        };

        match extract_invoice(&input, &config, &llm).await {
            Ok(result) => {
                if let Err(err) = actions::run_after_extraction(&config, &result, &input.bytes) {
                    tracing::error!("Post-processing aborted for {}: {err:#}", input.path);
                    if input_count == 1 {
                        return Err(err);
                    }
                }
                if streaming {
                    write_ndjson_record(&result)?;
                } else {
                    results.push(result);
                }
                success_count += 1;
            }
            Err(error) => {
                tracing::error!("Failed to process {}: {}", input.path, error);
                if input_count == 1 {
                    return Err(error);
                }
            }
        }
    }

    if !streaming {
        print!("{}", render_results(&results, &config, &output_options)?);
    }

    // Batch with zero successes must surface a non-zero exit so wrappers like
    // `invo *.pdf || handle` don't see false success and archive empty output.
    // Single-file failures already propagated above as anyhow::Err.
    if input_count > 1 && success_count == 0 {
        eprintln!(
            "All {input_count} files failed extraction; exiting non-zero. See log for per-file errors."
        );
        std::process::exit(2);
    }

    Ok(())
}

fn init_logging(verbose: u8) {
    let log_level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();
}

fn handle_command(command: &Commands, cli: &Cli) -> Result<()> {
    match command {
        Commands::Init { force } => handle_init(cli.config.as_ref(), *force),
        Commands::Config { command } => {
            handle_config_command(command, &config_command_options(cli))
        }
        Commands::AgentContext => handle_agent_context(&agent_context_options(cli)),
        Commands::Completions { shell } => handle_completions(*shell),
    }
}

fn handle_completions(shell: Shell) -> Result<()> {
    use std::io;

    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());

    Ok(())
}

fn input_options(cli: &Cli) -> InputOptions {
    InputOptions {
        files: cli.files.clone(),
        files_from: cli.files_from.clone(),
    }
}

fn output_options(cli: &Cli) -> OutputOptions {
    OutputOptions {
        format: cli.format,
        json: cli.json,
        json_array: cli.json_array,
        pretty: cli.pretty,
    }
}

fn config_command_options(cli: &Cli) -> ConfigCommandOptions {
    ConfigCommandOptions {
        config: cli.config.clone(),
        json: cli.json,
    }
}

fn agent_context_options(cli: &Cli) -> AgentContextOptions {
    AgentContextOptions {
        config: cli.config.clone(),
        json: cli.json,
    }
}
