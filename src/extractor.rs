use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chrono::Utc;
use std::time::Instant;

use crate::config::{Config, OcrConfig, Provider, ProviderName};
use crate::input::InvoiceInput;
use crate::invoice::{ExtractionMeta, ExtractionResult, FileInfo};
use crate::llm::LlmContext;
use crate::pdf::{extract_text, MIN_MEANINGFUL_TEXT_LENGTH};
use crate::prompt::{extraction_system_prompt, extraction_user_message};

pub async fn extract_invoice(
    input: &InvoiceInput,
    config: &Config,
    llm: &LlmContext,
) -> Result<ExtractionResult> {
    tracing::info!("Processing: {}", input.path);
    let start = Instant::now();

    let mut pdf_result = extract_text(&input.bytes)
        .with_context(|| format!("Failed to extract text from PDF: {}", input.path))?;

    if pdf_result.text.len() < MIN_MEANINGFUL_TEXT_LENGTH {
        let resolved = resolve_ocr(&config.processing.ocr, llm.provider())?;
        match resolved {
            ResolvedOcr::Disabled => bail!(
                "PDF appears scanned (only {} chars of native text). Configure processing.ocr.mode to enable OCR (mistral | anthropic | openai | ollama).",
                pdf_result.text.len()
            ),
            other => {
                tracing::info!("Falling back to OCR via {}", other.label());
                pdf_result.text = other
                    .run(llm.http(), &input.bytes)
                    .await
                    .context("OCR fallback failed")?;
                pdf_result.ocr_used = true;
            }
        }
    }

    if pdf_result.text.is_empty() {
        bail!("No text extracted from PDF");
    }

    tracing::debug!(
        "Using {} chars from {} pages (ocr_used={})",
        pdf_result.text.len(),
        pdf_result.page_count,
        pdf_result.ocr_used
    );

    let llm_result = llm
        .extract(
            config,
            extraction_system_prompt(&config.identity, &config.vendors),
            extraction_user_message(&pdf_result.text, config.llm.extraction.max_pdf_text_chars),
        )
        .await?;

    let result = ExtractionResult {
        file: FileInfo {
            path: input.path.clone(),
            original_name: input.original_name.clone(),
            hash: pdf_result.file_hash,
            processed_at: Utc::now().to_rfc3339(),
        },
        invoice: llm_result.invoice,
        extraction: ExtractionMeta {
            provider: llm_result.provider,
            model: llm_result.model,
            ocr_used: pdf_result.ocr_used,
            processing_time_ms: start.elapsed().as_millis() as u64,
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

// --- OCR fallback ----------------------------------------------------------
//
// Four backends in one place. Mistral / Anthropic / OpenAI accept PDF bytes
// inline (no client-side rasterization). Ollama vision models accept images
// only, so its branch is feature-gated behind `ollama-ocr` which pulls in
// PDFium. Resolution is feature-independent — only the actual OCR call fails
// at runtime when the feature is missing, with an actionable rebuild hint.

const EXTRACT_PROMPT: &str =
    "Extract all text from this document verbatim as markdown. Preserve line items, totals, and dates. Do not summarize or omit anything.";

const DEFAULT_MISTRAL_OCR_MODEL: &str = "mistral-ocr-latest";
const DEFAULT_ANTHROPIC_OCR_MODEL: &str = "claude-3-5-sonnet-latest";
const DEFAULT_OPENAI_OCR_MODEL: &str = "gpt-4o";
const DEFAULT_OLLAMA_OCR_MODEL: &str = "llama3.2-vision";
const DEFAULT_OLLAMA_OCR_URL: &str = "http://localhost:11434";

/// Authentication shape per provider. Cheap to pass by value.
enum Auth<'a> {
    Bearer(&'a str),
    AnthropicKey(&'a str),
    #[cfg_attr(not(feature = "ollama-ocr"), allow(dead_code))]
    None,
}

/// Shared POST + error-context machinery. Replaces 8 duplicated lines × 4 fns.
async fn ocr_post(
    http: &reqwest::Client,
    url: &str,
    auth: Auth<'_>,
    payload: &serde_json::Value,
    label: &str,
) -> Result<serde_json::Value> {
    let mut req = http.post(url).json(payload);
    req = match auth {
        Auth::Bearer(k) => req.bearer_auth(k),
        Auth::AnthropicKey(k) => req
            .header("x-api-key", k)
            .header("anthropic-version", "2023-06-01"),
        Auth::None => req,
    };
    req.send()
        .await
        .with_context(|| format!("{label} OCR request failed (network)"))?
        .error_for_status()
        .with_context(|| format!("{label} OCR returned an error status"))?
        .json()
        .await
        .with_context(|| format!("{label} OCR returned malformed JSON"))
}

#[derive(Debug)]
enum ResolvedOcr {
    Disabled,
    Mistral { model: String, api_key: String },
    Anthropic { model: String, api_key: String },
    OpenAi { model: String, api_key: String },
    Ollama { model: String, base_url: String },
}

impl ResolvedOcr {
    fn label(&self) -> String {
        match self {
            ResolvedOcr::Disabled => "disabled".into(),
            ResolvedOcr::Mistral { model, .. } => format!("mistral ({})", model),
            ResolvedOcr::Anthropic { model, .. } => format!("anthropic ({})", model),
            ResolvedOcr::OpenAi { model, .. } => format!("openai ({})", model),
            ResolvedOcr::Ollama { model, base_url } => {
                format!("ollama ({} @ {})", model, base_url)
            }
        }
    }

    async fn run(self, http: &reqwest::Client, bytes: &[u8]) -> Result<String> {
        match self {
            ResolvedOcr::Disabled => bail!("OCR is disabled (internal: caller should branch)"),
            ResolvedOcr::Mistral { model, api_key } => {
                ocr_via_mistral(http, bytes, &model, &api_key).await
            }
            ResolvedOcr::Anthropic { model, api_key } => {
                ocr_via_anthropic(http, bytes, &model, &api_key).await
            }
            ResolvedOcr::OpenAi { model, api_key } => {
                ocr_via_openai(http, bytes, &model, &api_key).await
            }
            ResolvedOcr::Ollama { model, base_url } => {
                ocr_via_ollama(http, bytes, &model, &base_url).await
            }
        }
    }
}

fn resolve_ocr(ocr: &OcrConfig, extractor: &Provider) -> Result<ResolvedOcr> {
    match ocr {
        OcrConfig::None => Ok(ResolvedOcr::Disabled),

        OcrConfig::Mistral { model, api_key_env } => Ok(ResolvedOcr::Mistral {
            model: model.clone(),
            api_key: read_env(api_key_env)?,
        }),

        OcrConfig::Same => resolve_from_extractor(extractor),

        OcrConfig::Llm {
            name,
            model,
            api_key_env,
            base_url,
        } => resolve_llm(*name, model, api_key_env.as_deref(), base_url.as_deref()),
    }
}

fn resolve_from_extractor(extractor: &Provider) -> Result<ResolvedOcr> {
    match extractor.name {
        ProviderName::Mistral => Ok(ResolvedOcr::Mistral {
            model: nonempty_or(&extractor.model, DEFAULT_MISTRAL_OCR_MODEL),
            api_key: read_env(
                extractor
                    .api_key_env
                    .as_deref()
                    .unwrap_or("MISTRAL_API_KEY"),
            )?,
        }),
        ProviderName::Anthropic => Ok(ResolvedOcr::Anthropic {
            model: nonempty_or(&extractor.model, DEFAULT_ANTHROPIC_OCR_MODEL),
            api_key: read_env(
                extractor
                    .api_key_env
                    .as_deref()
                    .unwrap_or("ANTHROPIC_API_KEY"),
            )?,
        }),
        ProviderName::OpenAI => Ok(ResolvedOcr::OpenAi {
            model: nonempty_or(&extractor.model, DEFAULT_OPENAI_OCR_MODEL),
            api_key: read_env(extractor.api_key_env.as_deref().unwrap_or("OPENAI_API_KEY"))?,
        }),
        ProviderName::Ollama => Ok(ResolvedOcr::Ollama {
            model: nonempty_or(&extractor.model, DEFAULT_OLLAMA_OCR_MODEL),
            base_url: extractor
                .base_url
                .clone()
                .unwrap_or_else(|| DEFAULT_OLLAMA_OCR_URL.to_string()),
        }),
    }
}

fn resolve_llm(
    name: ProviderName,
    model: &str,
    api_key_env: Option<&str>,
    base_url: Option<&str>,
) -> Result<ResolvedOcr> {
    match name {
        ProviderName::Mistral => Ok(ResolvedOcr::Mistral {
            model: nonempty_or(model, DEFAULT_MISTRAL_OCR_MODEL),
            api_key: read_env(api_key_env.unwrap_or("MISTRAL_API_KEY"))?,
        }),
        ProviderName::Anthropic => Ok(ResolvedOcr::Anthropic {
            model: nonempty_or(model, DEFAULT_ANTHROPIC_OCR_MODEL),
            api_key: read_env(api_key_env.unwrap_or("ANTHROPIC_API_KEY"))?,
        }),
        ProviderName::OpenAI => Ok(ResolvedOcr::OpenAi {
            model: nonempty_or(model, DEFAULT_OPENAI_OCR_MODEL),
            api_key: read_env(api_key_env.unwrap_or("OPENAI_API_KEY"))?,
        }),
        ProviderName::Ollama => Ok(ResolvedOcr::Ollama {
            model: nonempty_or(model, DEFAULT_OLLAMA_OCR_MODEL),
            base_url: base_url.unwrap_or(DEFAULT_OLLAMA_OCR_URL).to_string(),
        }),
    }
}

fn nonempty_or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn read_env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| {
        anyhow!(
            "OCR API key environment variable `{}` is not set. Export it or set processing.ocr.api_key_env to a different variable.",
            name
        )
    })
}

// --- Mistral OCR -----------------------------------------------------------

async fn ocr_via_mistral(
    http: &reqwest::Client,
    bytes: &[u8],
    model: &str,
    api_key: &str,
) -> Result<String> {
    let data_uri = format!("data:application/pdf;base64,{}", B64.encode(bytes));
    let payload = serde_json::json!({
        "model": model,
        "document": { "type": "document_url", "document_url": data_uri },
    });
    let resp = ocr_post(
        http,
        "https://api.mistral.ai/v1/ocr",
        Auth::Bearer(api_key),
        &payload,
        "Mistral",
    )
    .await?;

    let pages = resp["pages"]
        .as_array()
        .context("Mistral OCR response missing 'pages' array")?;
    let text = pages
        .iter()
        .filter_map(|p| p["markdown"].as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    if text.is_empty() {
        bail!("Mistral OCR returned no text (all pages had empty markdown)");
    }
    Ok(text)
}

// --- Anthropic OCR (native PDF document content block) --------------------

async fn ocr_via_anthropic(
    http: &reqwest::Client,
    bytes: &[u8],
    model: &str,
    api_key: &str,
) -> Result<String> {
    let payload = serde_json::json!({
        "model": model,
        "max_tokens": 8192,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "document",
                    "source": {
                        "type": "base64",
                        "media_type": "application/pdf",
                        "data": B64.encode(bytes),
                    }
                },
                { "type": "text", "text": EXTRACT_PROMPT }
            ]
        }]
    });
    let resp = ocr_post(
        http,
        "https://api.anthropic.com/v1/messages",
        Auth::AnthropicKey(api_key),
        &payload,
        "Anthropic",
    )
    .await?;

    let content = resp["content"]
        .as_array()
        .context("Anthropic OCR response missing 'content' array")?;
    let text = content
        .iter()
        .filter(|c| c["type"].as_str() == Some("text"))
        .filter_map(|c| c["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() {
        bail!("Anthropic OCR returned no text content");
    }
    Ok(text)
}

// --- OpenAI OCR (Responses API + input_file) -------------------------------
//
// Uses /v1/responses (NOT the chat completions endpoint that Rig uses for
// extraction). Responses API natively accepts base64 PDFs via input_file
// content, no Files-API roundtrip required.

async fn ocr_via_openai(
    http: &reqwest::Client,
    bytes: &[u8],
    model: &str,
    api_key: &str,
) -> Result<String> {
    let payload = serde_json::json!({
        "model": model,
        "input": [{
            "role": "user",
            "content": [
                {
                    "type": "input_file",
                    "filename": "document.pdf",
                    "file_data": format!("data:application/pdf;base64,{}", B64.encode(bytes)),
                },
                { "type": "input_text", "text": EXTRACT_PROMPT }
            ]
        }]
    });
    let resp = ocr_post(
        http,
        "https://api.openai.com/v1/responses",
        Auth::Bearer(api_key),
        &payload,
        "OpenAI",
    )
    .await?;

    // Prefer top-level convenience field when present; fall back to walking
    // the structured `output[].content[].text` shape so we work across
    // Responses-API revisions.
    let text = if let Some(s) = resp["output_text"].as_str() {
        s.to_string()
    } else {
        resp["output"]
            .as_array()
            .context("OpenAI OCR response missing 'output' array")?
            .iter()
            .flat_map(|item| item["content"].as_array().cloned().unwrap_or_default())
            .filter(|c| {
                let t = c["type"].as_str();
                t == Some("output_text") || t == Some("text")
            })
            .filter_map(|c| c["text"].as_str().map(str::to_string))
            .collect::<Vec<_>>()
            .join("\n")
    };
    if text.is_empty() {
        bail!("OpenAI OCR returned no text content");
    }
    Ok(text)
}

// --- Ollama OCR (vision models + client-side rasterization) ---------------
//
// Ollama vision models accept images only. Rasterization uses PDFium via
// `pdfium-render`, which requires a native PDFium library. Gated behind the
// `ollama-ocr` Cargo feature to keep default builds dep-free and
// cross-platform-clean. Without the feature, the call returns an actionable
// rebuild hint at runtime.

#[cfg(feature = "ollama-ocr")]
async fn ocr_via_ollama(
    http: &reqwest::Client,
    bytes: &[u8],
    model: &str,
    base_url: &str,
) -> Result<String> {
    let bytes = bytes.to_vec();
    let images = tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;
        use pdfium_render::prelude::*;

        // Pdfium is !Send — bind, load, render all inside this blocking task.
        let bindings = Pdfium::bind_to_system_library()
            .or_else(|_| {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            })
            .context(
                "Could not load PDFium dynamic library. Install libpdfium for your platform (https://github.com/bblanchon/pdfium-binaries) and ensure it is on the dynamic-loader path, or place the platform library next to the invo binary.",
            )?;
        let pdfium = Pdfium::new(bindings);

        let doc = pdfium
            .load_pdf_from_byte_vec(bytes, None)
            .context("PDFium could not parse PDF bytes")?;
        let render_config = PdfRenderConfig::new().set_target_width(1536);

        let mut images = Vec::new();
        for page in doc.pages().iter() {
            let bitmap = page
                .render_with_config(&render_config)
                .context("PDFium failed to render page")?;
            let dyn_image = bitmap.as_image();
            let rgba = dyn_image.to_rgba8();
            let (w, h) = rgba.dimensions();
            let mut png_bytes = Vec::new();
            PngEncoder::new(&mut png_bytes)
                .write_image(rgba.as_raw(), w, h, image::ExtendedColorType::Rgba8)
                .context("PNG encoding failed")?;
            images.push(B64.encode(&png_bytes));
        }
        Ok(images)
    })
    .await
    .context("PDFium rasterization task panicked")??;

    if images.is_empty() {
        bail!("PDFium produced no rendered pages");
    }

    let payload = serde_json::json!({
        "model": model,
        "stream": false,
        "messages": [{
            "role": "user",
            "content": EXTRACT_PROMPT,
            "images": images,
        }]
    });

    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let resp = ocr_post(http, &url, Auth::None, &payload, "Ollama").await?;

    let text = resp["message"]["content"]
        .as_str()
        .context("Ollama OCR response missing message.content")?;
    if text.is_empty() {
        bail!("Ollama OCR returned empty message content");
    }
    Ok(text.to_string())
}

#[cfg(not(feature = "ollama-ocr"))]
async fn ocr_via_ollama(
    _http: &reqwest::Client,
    _bytes: &[u8],
    _model: &str,
    _base_url: &str,
) -> Result<String> {
    bail!(
        "Ollama OCR requires the `ollama-ocr` Cargo feature (PDF rasterization via PDFium). Rebuild with `cargo install --features ollama-ocr` or use mode = mistral | anthropic | llm (with name: openai) instead."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mistral_provider() -> Provider {
        Provider {
            name: ProviderName::Mistral,
            enabled: true,
            model: "mistral-large-latest".to_string(),
            base_url: None,
            api_key_env: Some("MISTRAL_API_KEY".to_string()),
        }
    }

    fn ollama_provider() -> Provider {
        Provider {
            name: ProviderName::Ollama,
            enabled: true,
            model: "qwen2.5:1.5b".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            api_key_env: None,
        }
    }

    // Each test sets its own env var; tests that share a key name run
    // serially via the SAFETY_TEST_LOCK to avoid cross-test races since
    // cargo test uses threads by default.
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce()>(key: &str, value: Option<&str>, f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var(key).ok();
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
        f();
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn resolve_ocr_none_is_disabled() {
        let resolved =
            resolve_ocr(&OcrConfig::None, &ollama_provider()).expect("None resolves cleanly");
        assert!(matches!(resolved, ResolvedOcr::Disabled));
    }

    #[test]
    fn resolve_ocr_mistral_uses_env_api_key() {
        with_env("INVO_TEST_MISTRAL_KEY_1", Some("k-abc"), || {
            let cfg = OcrConfig::Mistral {
                model: "mistral-ocr-latest".to_string(),
                api_key_env: "INVO_TEST_MISTRAL_KEY_1".to_string(),
            };
            let resolved = resolve_ocr(&cfg, &ollama_provider()).expect("mistral resolves");
            match resolved {
                ResolvedOcr::Mistral { model, api_key } => {
                    assert_eq!(model, "mistral-ocr-latest");
                    assert_eq!(api_key, "k-abc");
                }
                _ => panic!("expected Mistral variant"),
            }
        });
    }

    #[test]
    fn resolve_ocr_mistral_errors_when_env_unset() {
        with_env("INVO_TEST_MISTRAL_KEY_MISSING", None, || {
            let cfg = OcrConfig::Mistral {
                model: "mistral-ocr-latest".to_string(),
                api_key_env: "INVO_TEST_MISTRAL_KEY_MISSING".to_string(),
            };
            let err = resolve_ocr(&cfg, &ollama_provider()).expect_err("missing env should fail");
            assert!(err.to_string().contains("INVO_TEST_MISTRAL_KEY_MISSING"));
        });
    }

    #[test]
    fn resolve_ocr_same_routes_to_mistral_when_extractor_is_mistral() {
        with_env("MISTRAL_API_KEY", Some("k-same"), || {
            let resolved =
                resolve_ocr(&OcrConfig::Same, &mistral_provider()).expect("same resolves");
            match resolved {
                ResolvedOcr::Mistral { model, api_key } => {
                    assert_eq!(model, "mistral-large-latest");
                    assert_eq!(api_key, "k-same");
                }
                _ => panic!("expected Mistral variant"),
            }
        });
    }

    fn anthropic_provider() -> Provider {
        Provider {
            name: ProviderName::Anthropic,
            enabled: true,
            model: "claude-3-5-sonnet-latest".to_string(),
            base_url: None,
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
        }
    }

    fn openai_provider() -> Provider {
        Provider {
            name: ProviderName::OpenAI,
            enabled: true,
            model: "gpt-4o".to_string(),
            base_url: None,
            api_key_env: Some("OPENAI_API_KEY".to_string()),
        }
    }

    #[test]
    fn resolve_ocr_same_routes_to_anthropic_when_extractor_is_anthropic() {
        with_env("ANTHROPIC_API_KEY", Some("k-ant"), || {
            let resolved =
                resolve_ocr(&OcrConfig::Same, &anthropic_provider()).expect("same+anthropic");
            match resolved {
                ResolvedOcr::Anthropic { model, api_key } => {
                    assert_eq!(model, "claude-3-5-sonnet-latest");
                    assert_eq!(api_key, "k-ant");
                }
                _ => panic!("expected Anthropic variant"),
            }
        });
    }

    #[test]
    fn resolve_ocr_same_routes_to_openai_when_extractor_is_openai() {
        with_env("OPENAI_API_KEY", Some("k-oai"), || {
            let resolved = resolve_ocr(&OcrConfig::Same, &openai_provider()).expect("same+openai");
            match resolved {
                ResolvedOcr::OpenAi { model, api_key } => {
                    assert_eq!(model, "gpt-4o");
                    assert_eq!(api_key, "k-oai");
                }
                _ => panic!("expected OpenAi variant"),
            }
        });
    }

    #[test]
    fn resolve_ocr_same_routes_to_ollama_when_extractor_is_ollama() {
        let resolved = resolve_ocr(&OcrConfig::Same, &ollama_provider()).expect("same+ollama");
        match resolved {
            ResolvedOcr::Ollama { model, base_url } => {
                assert_eq!(model, "qwen2.5:1.5b");
                assert_eq!(base_url, "http://localhost:11434");
            }
            _ => panic!("expected Ollama variant"),
        }
    }

    #[test]
    fn resolve_ocr_llm_routes_to_mistral_when_name_is_mistral() {
        with_env("INVO_TEST_LLM_MISTRAL_KEY", Some("k-llm"), || {
            let cfg = OcrConfig::Llm {
                name: ProviderName::Mistral,
                model: "mistral-ocr-latest".to_string(),
                api_key_env: Some("INVO_TEST_LLM_MISTRAL_KEY".to_string()),
                base_url: None,
            };
            let resolved = resolve_ocr(&cfg, &ollama_provider()).expect("llm+mistral");
            assert!(matches!(resolved, ResolvedOcr::Mistral { .. }));
        });
    }

    #[test]
    fn resolve_ocr_llm_routes_to_anthropic_with_explicit_env() {
        with_env("INVO_TEST_LLM_ANTHROPIC_KEY", Some("k-llm-ant"), || {
            let cfg = OcrConfig::Llm {
                name: ProviderName::Anthropic,
                model: "claude-3-5-sonnet-latest".to_string(),
                api_key_env: Some("INVO_TEST_LLM_ANTHROPIC_KEY".to_string()),
                base_url: None,
            };
            let resolved = resolve_ocr(&cfg, &mistral_provider()).expect("llm+anthropic");
            assert!(matches!(resolved, ResolvedOcr::Anthropic { .. }));
        });
    }

    #[test]
    fn resolve_ocr_llm_routes_to_openai_with_explicit_env() {
        with_env("INVO_TEST_LLM_OPENAI_KEY", Some("k-llm-oai"), || {
            let cfg = OcrConfig::Llm {
                name: ProviderName::OpenAI,
                model: "gpt-4o".to_string(),
                api_key_env: Some("INVO_TEST_LLM_OPENAI_KEY".to_string()),
                base_url: None,
            };
            let resolved = resolve_ocr(&cfg, &mistral_provider()).expect("llm+openai");
            assert!(matches!(resolved, ResolvedOcr::OpenAi { .. }));
        });
    }

    #[test]
    fn resolve_ocr_llm_routes_to_ollama_with_custom_base_url() {
        let cfg = OcrConfig::Llm {
            name: ProviderName::Ollama,
            model: "llama3.2-vision".to_string(),
            api_key_env: None,
            base_url: Some("http://example:11434".to_string()),
        };
        let resolved = resolve_ocr(&cfg, &mistral_provider()).expect("llm+ollama");
        match resolved {
            ResolvedOcr::Ollama { model, base_url } => {
                assert_eq!(model, "llama3.2-vision");
                assert_eq!(base_url, "http://example:11434");
            }
            _ => panic!("expected Ollama variant"),
        }
    }

    #[cfg(not(feature = "ollama-ocr"))]
    #[tokio::test]
    async fn ollama_ocr_without_feature_returns_actionable_error() {
        let http = test_http_client();
        let err = ocr_via_ollama(&http, b"%PDF", "llama3.2-vision", "http://localhost:11434")
            .await
            .expect_err("should fail without feature");
        let msg = err.to_string();
        assert!(msg.contains("ollama-ocr"));
        assert!(msg.contains("Rebuild") || msg.contains("rebuild"));
    }

    #[test]
    fn ocr_config_default_is_none() {
        let cfg = OcrConfig::default();
        assert!(matches!(cfg, OcrConfig::None));
    }

    // Live smoke test against api.mistral.ai. Requires MISTRAL_API_KEY in the
    // environment and network access. Run with:
    //     cargo test --bin invo -- --ignored mistral_ocr_extracts_text_from_sample_pdf
    // Uses a normal text-based sample PDF because the OCR endpoint accepts any
    // PDF; for a true scanned-PDF check, replace with an image-only fixture.
    fn sample_pdf_bytes() -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/claude/Receipt-2238-2456-7832.pdf");
        std::fs::read(&path).expect("sample PDF should be readable")
    }

    fn assert_meaningful_text(text: &str, label: &str) {
        assert!(!text.is_empty(), "{} returned empty text", label);
        assert!(
            text.len() > 50,
            "{} returned only {} chars — expected meaningful invoice text",
            label,
            text.len()
        );
    }

    fn test_http_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("test http client")
    }

    // Live OCR smoke tests against real provider APIs. Each test is gated on
    // its provider's API key env var. Run any single one with:
    //     cargo test --bin invo -- --ignored <test_name>

    #[ignore]
    #[tokio::test]
    async fn mistral_ocr_extracts_text_from_sample_pdf() {
        let api_key =
            std::env::var("MISTRAL_API_KEY").expect("set MISTRAL_API_KEY to run this live test");
        let text = ocr_via_mistral(
            &test_http_client(),
            &sample_pdf_bytes(),
            DEFAULT_MISTRAL_OCR_MODEL,
            &api_key,
        )
        .await
        .expect("Mistral OCR should succeed for a valid PDF");
        assert_meaningful_text(&text, "Mistral OCR");
    }

    #[ignore]
    #[tokio::test]
    async fn anthropic_ocr_extracts_text_from_sample_pdf() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("set ANTHROPIC_API_KEY to run this live test");
        let text = ocr_via_anthropic(
            &test_http_client(),
            &sample_pdf_bytes(),
            DEFAULT_ANTHROPIC_OCR_MODEL,
            &api_key,
        )
        .await
        .expect("Anthropic OCR should succeed for a valid PDF");
        assert_meaningful_text(&text, "Anthropic OCR");
    }

    #[ignore]
    #[tokio::test]
    async fn openai_ocr_extracts_text_from_sample_pdf() {
        let api_key =
            std::env::var("OPENAI_API_KEY").expect("set OPENAI_API_KEY to run this live test");
        let text = ocr_via_openai(
            &test_http_client(),
            &sample_pdf_bytes(),
            DEFAULT_OPENAI_OCR_MODEL,
            &api_key,
        )
        .await
        .expect("OpenAI OCR should succeed for a valid PDF");
        assert_meaningful_text(&text, "OpenAI OCR");
    }

    // Ollama live test: requires a running Ollama instance with a vision
    // model, AND the `ollama-ocr` feature enabled at build time. Run with:
    //     cargo test --bin invo --features ollama-ocr -- --ignored ollama_ocr_
    #[cfg(feature = "ollama-ocr")]
    #[ignore]
    #[tokio::test]
    async fn ollama_ocr_extracts_text_from_sample_pdf() {
        let text = ocr_via_ollama(
            &test_http_client(),
            &sample_pdf_bytes(),
            DEFAULT_OLLAMA_OCR_MODEL,
            DEFAULT_OLLAMA_OCR_URL,
        )
        .await
        .expect("Ollama OCR should succeed for a valid PDF");
        assert_meaningful_text(&text, "Ollama OCR");
    }
}
