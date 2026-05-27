use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::config::{Config, OutputFormat};
use crate::invoice::ExtractionResult;

#[derive(Debug, Clone, Copy)]
pub struct OutputOptions {
    pub format: Option<OutputFormat>,
    pub json: bool,
    pub json_array: bool,
    pub pretty: bool,
}

pub fn render_results(
    results: &[ExtractionResult],
    config: &Config,
    options: &OutputOptions,
) -> Result<String> {
    if results.is_empty() {
        return Ok(String::new());
    }

    let format = output_format(config, options)?;

    match format {
        OutputFormat::Json => render_json_results(results, options),
        OutputFormat::Yaml => render_yaml_results(results),
        OutputFormat::Csv => render_csv_results(results),
    }
}

pub fn output_format(config: &Config, options: &OutputOptions) -> Result<OutputFormat> {
    if options.json || options.json_array {
        if let Some(format) = options
            .format
            .filter(|format| *format != OutputFormat::Json)
        {
            return Err(anyhow!(
                "--json and --json-array cannot be combined with --format {:?}",
                format
            ));
        }
        return Ok(OutputFormat::Json);
    }

    Ok(options.format.unwrap_or(config.output.format))
}

fn render_json_results(results: &[ExtractionResult], options: &OutputOptions) -> Result<String> {
    let output = if options.json_array {
        if options.pretty {
            serde_json::to_string_pretty(results)?
        } else {
            serde_json::to_string(results)?
        }
    } else if results.len() == 1 {
        if options.json && !options.pretty {
            serde_json::to_string(&results[0])?
        } else {
            serde_json::to_string_pretty(&results[0])?
        }
    } else {
        results
            .iter()
            .map(serde_json::to_string)
            .collect::<serde_json::Result<Vec<_>>>()?
            .join("\n")
    };

    Ok(with_newline(output))
}

fn render_yaml_results(results: &[ExtractionResult]) -> Result<String> {
    let output = results
        .iter()
        .map(serde_yaml::to_string)
        .collect::<serde_yaml::Result<Vec<_>>>()?
        .join("---\n");

    Ok(with_newline(output))
}

fn render_csv_results(results: &[ExtractionResult]) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(["vendor", "date", "total", "currency", "invoice_number"])?;

    for result in results {
        writer.write_record([
            defuse_formula(&result.invoice.vendor),
            defuse_formula(&result.invoice.invoice_date.to_string()),
            defuse_formula(&format_total(result.invoice.total_amount)),
            defuse_formula(&result.invoice.currency),
            defuse_formula(result.invoice.invoice_number.as_deref().unwrap_or("")),
        ])?;
    }

    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

// Prefix fields whose first byte is in the spreadsheet-formula trigger set with
// a leading single quote so Excel/Numbers/LibreOffice treat them as text.
// Covers OWASP CSV-injection chars =, +, -, @, plus tab and CR which some
// engines also honour as formula intros.
fn defuse_formula(value: &str) -> String {
    match value.as_bytes().first() {
        Some(b'=' | b'+' | b'-' | b'@' | b'\t' | b'\r') => {
            let mut defused = String::with_capacity(value.len() + 1);
            defused.push('\'');
            defused.push_str(value);
            defused
        }
        _ => value.to_string(),
    }
}

// NaN/Inf serialise as "NaN"/"inf" via `{}` which is technically valid CSV but
// useless to spreadsheets. Emit an empty field instead so downstream pipelines
// see "no number here" rather than a text label masquerading as a total.
fn format_total(total: f64) -> String {
    if total.is_finite() {
        total.to_string()
    } else {
        String::new()
    }
}

pub(crate) fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// True when stdout should be emitted as one compact JSON object per line
/// (NDJSON), record-by-record as extractions complete. Determined by input
/// count so the shape of stdout is stable regardless of how many extractions
/// succeed.
pub fn is_ndjson_stream(config: &Config, options: &OutputOptions, input_count: usize) -> bool {
    if options.json_array {
        return false;
    }
    if input_count <= 1 {
        return false;
    }
    matches!(output_format(config, options), Ok(OutputFormat::Json))
}

/// Write a single ExtractionResult as one compact NDJSON line + flush so a
/// downstream pipeline consumer sees the record immediately.
pub fn write_ndjson_record(result: &ExtractionResult) -> Result<()> {
    use std::io::Write;
    let line = serde_json::to_string(result)?;
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(line.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

pub(crate) fn with_newline(mut output: String) -> String {
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoice::{ExtractionMeta, FileInfo, Invoice};
    use chrono::NaiveDate;
    use serde_json::Value as JsonValue;

    fn options() -> OutputOptions {
        OutputOptions {
            format: None,
            json: false,
            json_array: false,
            pretty: false,
        }
    }

    fn result(vendor: &str) -> ExtractionResult {
        ExtractionResult {
            file: FileInfo {
                path: format!("{vendor}.pdf"),
                original_name: format!("{vendor}.pdf"),
                hash: "hash".to_string(),
                processed_at: "2026-05-25T12:00:00Z".to_string(),
            },
            invoice: Invoice {
                vendor: vendor.to_string(),
                invoice_date: NaiveDate::from_ymd_opt(2026, 5, 25).expect("valid date"),
                total_amount: 12.5,
                currency: "USD".to_string(),
                invoice_number: None,
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
                items: Vec::new(),
                category: None,
                description: None,
                buyer_name: None,
                confidence: None,
            },
            extraction: ExtractionMeta {
                provider: "ollama".to_string(),
                model: "qwen2.5:1.5b".to_string(),
                ocr_used: false,
                processing_time_ms: 1,
            },
            identity_match: None,
        }
    }

    #[test]
    fn batch_json_renders_compact_ndjson_even_when_pretty_requested() {
        let mut options = options();
        options.pretty = true;
        let output = render_results(
            &[result("acme"), result("globex")],
            &Config::default(),
            &options,
        )
        .expect("batch json should render");

        let lines = output.trim_end().lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(!output.contains("\n  \"file\""));
        for line in lines {
            serde_json::from_str::<JsonValue>(line).expect("each NDJSON line should parse");
        }
    }

    #[test]
    fn json_array_renders_valid_batch_array() {
        let mut options = options();
        options.json_array = true;
        let output = render_results(
            &[result("acme"), result("globex")],
            &Config::default(),
            &options,
        )
        .expect("json array should render");

        let parsed = serde_json::from_str::<Vec<ExtractionResult>>(&output)
            .expect("batch array should parse");

        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn json_flag_renders_single_result_compact() {
        let mut options = options();
        options.json = true;
        let output = render_results(&[result("acme")], &Config::default(), &options)
            .expect("json should render");

        assert_eq!(output.lines().count(), 1);
        serde_json::from_str::<JsonValue>(&output).expect("compact json should parse");
    }

    #[test]
    fn single_default_json_remains_pretty_for_humans() {
        let output = render_results(&[result("acme")], &Config::default(), &options())
            .expect("json should render");

        assert!(output.contains("\n  \"file\""));
        serde_json::from_str::<JsonValue>(&output).expect("pretty json should parse");
    }

    #[test]
    fn json_flags_reject_non_json_format() {
        let mut options = options();
        options.json = true;
        options.format = Some(OutputFormat::Yaml);

        let error = output_format(&Config::default(), &options)
            .expect_err("json aliases should conflict with non-json format");

        assert!(error.to_string().contains("--json"));
    }

    #[test]
    fn is_ndjson_stream_true_only_for_batch_json() {
        let cfg = Config::default();

        // single input → not streaming, even with --json
        let mut opts = options();
        opts.json = true;
        assert!(!is_ndjson_stream(&cfg, &opts, 1));

        // batch + --json → streaming
        assert!(is_ndjson_stream(&cfg, &opts, 5));

        // --json-array always accumulates
        let mut opts = options();
        opts.json_array = true;
        assert!(!is_ndjson_stream(&cfg, &opts, 5));

        // default format on batch (json by default) → streaming
        assert!(is_ndjson_stream(&cfg, &options(), 3));

        // yaml/csv never stream via this path
        let mut opts = options();
        opts.format = Some(OutputFormat::Yaml);
        assert!(!is_ndjson_stream(&cfg, &opts, 5));
    }

    fn csv_options() -> OutputOptions {
        let mut opts = options();
        opts.format = Some(OutputFormat::Csv);
        opts
    }

    fn csv_result(invoice: Invoice) -> ExtractionResult {
        let mut base = result("placeholder");
        base.invoice = invoice;
        base
    }

    fn invoice_with(vendor: &str, total: f64, invoice_number: Option<&str>) -> Invoice {
        Invoice {
            vendor: vendor.to_string(),
            invoice_date: NaiveDate::from_ymd_opt(2026, 5, 25).expect("valid date"),
            total_amount: total,
            currency: "USD".to_string(),
            invoice_number: invoice_number.map(str::to_string),
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
            items: Vec::new(),
            category: None,
            description: None,
            buyer_name: None,
            confidence: None,
        }
    }

    #[test]
    fn csv_defuses_formula_leading_values() {
        // Each of these vendors would execute as a formula if Excel opened the
        // CSV. The render must prefix them with ' so they parse as text.
        for malicious in ["=cmd|/c calc!A0", "+1+1", "-7", "@SUM(A1:A2)"] {
            let output = render_results(
                &[csv_result(invoice_with(malicious, 1.0, None))],
                &Config::default(),
                &csv_options(),
            )
            .expect("csv should render");

            let data_line = output.lines().nth(1).expect("data row");
            assert!(
                data_line.contains(&format!("'{malicious}")),
                "vendor {malicious:?} should be prefixed with apostrophe, got: {data_line}"
            );
        }
    }

    #[test]
    fn csv_defuses_tab_and_cr_leading_values() {
        // Tab and CR also trigger formula evaluation in some spreadsheet
        // engines. Embed inside the vendor as the first byte.
        for (label, malicious) in [("tab", "\tfoo"), ("cr", "\rfoo")] {
            let output = render_results(
                &[csv_result(invoice_with(malicious, 1.0, None))],
                &Config::default(),
                &csv_options(),
            )
            .unwrap_or_else(|err| panic!("csv should render for {label}: {err}"));

            // csv writer will wrap CR/tab-bearing fields in quotes; the
            // defused payload appears as "'<char>foo". Search the raw bytes.
            assert!(
                output.contains(&format!("'{malicious}")),
                "{label}-leading vendor should be apostrophe-prefixed, got: {output:?}"
            );
        }
    }

    #[test]
    fn csv_quotes_embedded_commas_newlines_and_doubles_quotes() {
        let output = render_results(
            &[csv_result(invoice_with(
                "Acme, Inc.\n\"Subsidiary\"",
                1.0,
                None,
            ))],
            &Config::default(),
            &csv_options(),
        )
        .expect("csv should render");

        // Field with comma, newline, and quote must be wrapped in quotes with
        // the embedded quotes doubled. The csv crate handles all three.
        assert!(
            output.contains("\"Acme, Inc.\n\"\"Subsidiary\"\"\""),
            "field should be quoted with doubled quotes, got: {output:?}"
        );

        // Sanity: round-trip parse with a reader and confirm the vendor field.
        let mut reader = csv::Reader::from_reader(output.as_bytes());
        let row = reader
            .records()
            .next()
            .expect("one row")
            .expect("row should parse");
        assert_eq!(row.get(0), Some("Acme, Inc.\n\"Subsidiary\""));
    }

    #[test]
    fn csv_renders_empty_total_for_nan_and_infinity() {
        for total in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let output = render_results(
                &[csv_result(invoice_with("Acme", total, None))],
                &Config::default(),
                &csv_options(),
            )
            .expect("csv should render");

            let data_line = output.lines().nth(1).expect("data row");
            // Header: vendor,date,total,currency,invoice_number -> total is
            // column index 2. The csv parser is the source of truth.
            let mut reader = csv::Reader::from_reader(output.as_bytes());
            let row = reader
                .records()
                .next()
                .expect("one row")
                .expect("row should parse");
            assert_eq!(
                row.get(2),
                Some(""),
                "non-finite total should serialise empty (line was {data_line:?})"
            );
        }
    }

    #[test]
    fn ndjson_stream_concatenation_matches_batch_render() {
        // Concatenating per-record NDJSON output should byte-match what the
        // batch renderer produces today for the same inputs in the same order.
        let results = vec![result("acme"), result("globex"), result("initech")];

        let mut opts = options();
        opts.json = true;
        let batch = render_results(&results, &Config::default(), &opts).expect("batch render");

        let mut streamed = String::new();
        for r in &results {
            streamed.push_str(&serde_json::to_string(r).expect("compact json"));
            streamed.push('\n');
        }

        assert_eq!(streamed, batch);
    }
}
