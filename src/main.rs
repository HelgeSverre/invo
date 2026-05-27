mod actions;
mod agent_context;
mod cli;
mod config;
mod config_commands;
mod extractor;
mod input;
mod invoice;
mod llm;
mod output;
mod pdf;
mod prompt;

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use futures::StreamExt;

use agent_context::{handle_agent_context, AgentContextOptions};
use cli::{Cli, Commands};
use config::load_config;
use config_commands::{handle_config_command, handle_init, ConfigCommandOptions};
use extractor::extract_invoice;
use input::{collect_inputs, InputOptions};
use llm::LlmContext;
use output::{
    is_ndjson_stream, render_results, validate_output_flags, write_ndjson_record, OutputOptions,
};

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

    // Single-file mode preserves the anyhow-error propagation contract that
    // scripts depend on: process exits 1, error message goes to stderr via
    // anyhow chain. No need for stream/Arc overhead.
    if input_count == 1 {
        let input_spec = inputs
            .into_iter()
            .next()
            .expect("single-file input must exist");
        let input_path = input_spec.path.clone();
        let input = input_spec
            .read()
            .with_context(|| format!("Failed to read input: {input_path}"))?;
        let result = extract_invoice(&input, &config, &llm).await?;
        if !config.actions.is_empty() {
            // Single-file mode: fresh registry, no possible collisions.
            let registry = actions::new_destination_registry();
            actions::run_actions(&config.actions, &result, &input.bytes, &registry)
                .with_context(|| format!("Post-processing aborted for {}", input.path))?;
        }
        if streaming {
            write_ndjson_record(&result)?;
        } else {
            print!("{}", render_results(&[result], &config, &output_options)?);
        }
        return Ok(());
    }

    // Batch mode. Parallelism > 1 yields completion order (first-done-first-out)
    // for both streaming and buffered output. Each record's file.path + file.hash
    // self-identify which input it came from; consumers that need input order
    // should sort downstream.
    //
    // One destination registry shared across all workers — see decision-4 for
    // the first-writer-wins collision policy.
    let parallelism = config.processing.parallelism.max(1) as usize;
    let config = Arc::new(config);
    let llm = Arc::new(llm);
    let action_registry = actions::new_destination_registry();
    let mut results = Vec::new();
    let mut success_count: usize = 0;
    let mut failure_count: usize = 0;

    let mut stream = futures::stream::iter(inputs.into_iter().map(|input_spec| {
        let config = Arc::clone(&config);
        let llm = Arc::clone(&llm);
        let action_registry = action_registry.clone();
        async move {
            let input_path = input_spec.path.clone();
            let input = input_spec
                .read()
                .with_context(|| format!("Failed to read input: {input_path}"))?;
            let result = extract_invoice(&input, &config, &llm)
                .await
                .with_context(|| format!("Failed to process {}", input.path))?;
            if !config.actions.is_empty() {
                actions::run_actions(&config.actions, &result, &input.bytes, &action_registry)
                    .with_context(|| format!("Post-processing aborted for {}", input.path))?;
            }
            Ok::<_, anyhow::Error>(result)
        }
    }))
    .buffer_unordered(parallelism);

    while let Some(outcome) = stream.next().await {
        match outcome {
            Ok(result) => {
                if streaming {
                    write_ndjson_record(&result)?;
                } else {
                    results.push(result);
                }
                success_count += 1;
            }
            Err(error) => {
                tracing::error!("{error:#}");
                failure_count += 1;
            }
        }
    }

    if !streaming {
        print!("{}", render_results(&results, &config, &output_options)?);
    }

    // Single-file failures already propagated above as anyhow::Err. For batches
    // (input_count > 1), translate the success/failure split into a script-safe
    // exit code so wrappers like `invo *.pdf || handle` can react. See
    // `decide_batch_exit_code` for the 0/2/3 contract.
    if let Some(exit_code) = decide_batch_exit_code(input_count, success_count, failure_count) {
        eprintln!(
            "Batch: {success_count} succeeded, {failure_count} failed (of {input_count} files). See log for per-file errors."
        );
        std::process::exit(exit_code);
    }

    Ok(())
}

/// Script-safe exit-code contract for batch (multi-file) extraction:
///
/// - `0`: all inputs succeeded — no signal needed (returns `None`).
/// - `2`: every input failed. Wrappers piping output to archive paths must
///   not treat this as success.
/// - `3`: at least one input failed but at least one succeeded. Distinct
///   from 2 so scripts can choose to retry only the failed half or
///   accept partial results.
///
/// Single-file runs (`input_count <= 1`) are handled at the call site by
/// propagating the per-file error directly, so this returns `None` for them
/// too — there's nothing to *summarise* in single-file mode.
fn decide_batch_exit_code(
    input_count: usize,
    success_count: usize,
    failure_count: usize,
) -> Option<i32> {
    if input_count <= 1 || failure_count == 0 {
        return None;
    }
    if success_count == 0 {
        Some(2)
    } else {
        Some(3)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_file_returns_none_so_caller_propagates_anyhow_err() {
        // input_count <= 1 means single-file mode; the per-file error already
        // surfaces via the function's Result, so no batch exit code is needed.
        assert_eq!(decide_batch_exit_code(1, 1, 0), None);
        assert_eq!(decide_batch_exit_code(1, 0, 1), None);
        assert_eq!(decide_batch_exit_code(0, 0, 0), None);
    }

    #[test]
    fn batch_all_success_returns_none_so_process_exits_zero() {
        assert_eq!(decide_batch_exit_code(2, 2, 0), None);
        assert_eq!(decide_batch_exit_code(10, 10, 0), None);
    }

    #[test]
    fn batch_partial_failure_returns_exit_code_three() {
        // At least one success and at least one failure → exit 3 so scripts
        // can distinguish "retry the failures" from "everything broke".
        assert_eq!(decide_batch_exit_code(2, 1, 1), Some(3));
        assert_eq!(decide_batch_exit_code(10, 7, 3), Some(3));
    }

    #[test]
    fn batch_all_failure_returns_exit_code_two() {
        // Zero successes in a batch → exit 2 so 'invo *.pdf > out.json && mv'
        // doesn't archive empty output.
        assert_eq!(decide_batch_exit_code(2, 0, 2), Some(2));
        assert_eq!(decide_batch_exit_code(10, 0, 10), Some(2));
    }
}
