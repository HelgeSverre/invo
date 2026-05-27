use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

use crate::config::{OutputFormat, ProviderName};

#[derive(Parser)]
#[command(name = "invo")]
#[command(about = "Extract structured data from PDF receipts/invoices using LLMs")]
#[command(version)]
#[command(
    after_help = "Examples:\n  invo receipt.pdf\n  invo - < receipt.pdf\n  invo --files-from receipts.txt --json\n  invo \"receipts/**/*.pdf\" --json > receipts.ndjson\n  invo \"receipts/**/*.pdf\" --json-array > receipts.json\n  invo -c config.yaml config validate\n  invo config show --effective --json\n  invo agent-context --json"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// PDF file(s) to process (supports glob patterns). Use '-' to read PDF bytes from stdin.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Read newline-delimited file paths or globs from a file. Use '-' to read the list from stdin.
    #[arg(long, value_name = "PATH")]
    pub files_from: Option<PathBuf>,

    /// Output format (json, yaml, csv)
    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Emit machine-readable JSON where supported. For extraction, compact JSON; batches are NDJSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Emit one JSON array containing all successful results.
    #[arg(long)]
    pub json_array: bool,

    /// Pretty-print JSON for single-result human display.
    #[arg(long)]
    pub pretty: bool,

    /// Config file path
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// Use a specific LLM provider (ollama, openai, anthropic, mistral)
    #[arg(long, value_enum)]
    pub provider: Option<ProviderName>,

    /// Verbose output (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum ConfigCommand {
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
