use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DATE_FORMAT: &str = "%Y-%m-%d";

/// Line item on an invoice/receipt
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LineItem {
    #[serde(default)]
    pub sku: Option<String>,

    pub description: String,

    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>")]
    pub quantity: Option<f64>,

    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>")]
    pub unit_price: Option<f64>,

    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>")]
    pub total: Option<f64>,
}

/// Extracted invoice/receipt data from LLM
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Invoice {
    // Required fields
    pub vendor: String,

    #[serde(with = "date_format")]
    #[schemars(with = "String", description = "Invoice date in YYYY-MM-DD format")]
    pub invoice_date: NaiveDate,

    #[serde(deserialize_with = "string_or_f64::deserialize")]
    #[schemars(with = "f64", description = "Total amount as a number")]
    pub total_amount: f64,

    #[schemars(description = "ISO 4217 3-letter currency code (USD, EUR, NOK, etc.)")]
    pub currency: String,

    // Optional header fields
    #[serde(default)]
    pub invoice_number: Option<String>,

    #[serde(default, with = "option_date_format")]
    #[schemars(with = "Option<String>", description = "Due date in YYYY-MM-DD format")]
    pub due_date: Option<NaiveDate>,

    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>")]
    pub subtotal: Option<f64>,

    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>")]
    pub tax_amount: Option<f64>,

    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>")]
    pub tax_rate: Option<f64>,

    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>")]
    pub discount_amount: Option<f64>,

    // Vendor contact
    #[serde(default)]
    pub vendor_website: Option<String>,

    #[serde(default)]
    pub vendor_email: Option<String>,

    #[serde(default)]
    pub vendor_address: Option<String>,

    // Payment info
    #[serde(default)]
    pub payment_method: Option<String>,

    #[serde(default)]
    pub card_last_four: Option<String>,

    // Line items
    #[serde(default)]
    pub items: Vec<LineItem>,

    // Categorization
    #[serde(default)]
    pub category: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    // Buyer info (from LLM)
    #[serde(default)]
    pub buyer_name: Option<String>,

    // Confidence score from LLM
    #[serde(default, deserialize_with = "option_string_or_f64::deserialize")]
    #[schemars(with = "Option<f64>", description = "Confidence score 0.0-1.0")]
    pub confidence: Option<f64>,
}

/// Full extraction result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub file: FileInfo,
    pub invoice: Invoice,
    pub extraction: ExtractionMeta,

    #[serde(default)]
    pub identity_match: Option<IdentityMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub original_name: String,
    pub hash: String,
    pub processed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMeta {
    pub provider: String,
    pub model: String,
    pub ocr_used: bool,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityMatch {
    pub buyer_name: String,
    pub is_self: bool,
}

// Custom date serialization for YYYY-MM-DD format
mod date_format {
    use super::DATE_FORMAT;
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(DATE_FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)
    }
}

mod option_date_format {
    use super::DATE_FORMAT;
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_some(&d.format(DATE_FORMAT).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) if !s.is_empty() => NaiveDate::parse_from_str(&s, DATE_FORMAT)
                .map(Some)
                .map_err(serde::de::Error::custom),
            _ => Ok(None),
        }
    }
}

/// Strip common currency symbols/codes and parse a numeric string.
fn parse_loose_number(s: &str) -> Option<f64> {
    let cleaned = s
        .trim()
        .replace(',', ".")
        .replace(['$', '€', '£', '¥', '₹', ' '], "")
        .replace("kr", "")
        .replace("NOK", "")
        .replace("USD", "")
        .replace("EUR", "");
    cleaned.parse::<f64>().ok()
}

/// Deserializer that accepts both f64 and string representations of numbers
mod string_or_f64 {
    use super::parse_loose_number;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            Float(f64),
            String(String),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::Float(f) => Ok(f),
            StringOrFloat::String(s) => parse_loose_number(&s)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid number: {}", s))),
        }
    }
}

/// Deserializer for Option<f64> that accepts both f64 and string representations
mod option_string_or_f64 {
    use super::parse_loose_number;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            Float(f64),
            String(String),
        }

        match Option::<StringOrFloat>::deserialize(deserializer)? {
            None => Ok(None),
            Some(StringOrFloat::Float(f)) => Ok(Some(f)),
            Some(StringOrFloat::String(s)) if s.is_empty() => Ok(None),
            Some(StringOrFloat::String(s)) => parse_loose_number(&s)
                .map(Some)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid number: {}", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Output wrapper shape (TASK-27).
    //!
    //! These tests pin the JSON envelope that wraps the LLM-extracted invoice
    //! payload. They do NOT call any LLM — we construct an `ExtractionResult`
    //! directly with a stubbed `Invoice`. The shape we pin here is what
    //! downstream consumers (NDJSON streams, CSV converters, agent context)
    //! depend on.
    use super::*;
    use chrono::NaiveDate;
    use serde_json::Value;

    fn sample_extraction_result() -> ExtractionResult {
        ExtractionResult {
            file: FileInfo {
                path: "tests/fixtures/pdfs/single_page.pdf".to_string(),
                original_name: "single_page.pdf".to_string(),
                hash: "53975127489838ab".to_string(),
                processed_at: "2026-05-27T00:00:00+00:00".to_string(),
            },
            invoice: Invoice {
                vendor: "Acme Widgets Inc".to_string(),
                invoice_date: NaiveDate::from_ymd_opt(2026, 5, 27).unwrap(),
                total_amount: 123.45,
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
                items: Vec::new(),
                category: None,
                description: None,
                buyer_name: None,
                confidence: None,
            },
            extraction: ExtractionMeta {
                provider: "stub".to_string(),
                model: "stub-model".to_string(),
                ocr_used: false,
                processing_time_ms: 42,
            },
            identity_match: None,
        }
    }

    #[test]
    fn extraction_result_has_top_level_envelope_keys() {
        let json = serde_json::to_value(sample_extraction_result()).expect("serializes");
        let obj = json.as_object().expect("top level is object");

        // The envelope contract: these four keys exist, identity_match is
        // optional but present as null when absent.
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["extraction", "file", "identity_match", "invoice"]
        );
    }

    #[test]
    fn extraction_result_file_block_has_documented_fields() {
        let json = serde_json::to_value(sample_extraction_result()).expect("serializes");
        let file = json
            .get("file")
            .expect("file block")
            .as_object()
            .expect("object");

        let mut keys: Vec<&str> = file.keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(keys, vec!["hash", "original_name", "path", "processed_at"]);

        assert_eq!(file["hash"], Value::from("53975127489838ab"));
        assert_eq!(file["original_name"], Value::from("single_page.pdf"));
    }

    #[test]
    fn extraction_result_meta_block_has_documented_fields() {
        let json = serde_json::to_value(sample_extraction_result()).expect("serializes");
        let meta = json
            .get("extraction")
            .expect("extraction block")
            .as_object()
            .expect("object");

        let mut keys: Vec<&str> = meta.keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["model", "ocr_used", "processing_time_ms", "provider"]
        );
        assert_eq!(meta["ocr_used"], Value::from(false));
        assert_eq!(meta["processing_time_ms"], Value::from(42u64));
    }

    #[test]
    fn extraction_result_invoice_date_is_yyyy_mm_dd_string() {
        let json = serde_json::to_value(sample_extraction_result()).expect("serializes");
        let invoice = json
            .get("invoice")
            .expect("invoice block")
            .as_object()
            .expect("object");

        // Date contract: serialized as a string in YYYY-MM-DD form, never
        // as an array, object, or epoch number.
        assert_eq!(invoice["invoice_date"], Value::from("2026-05-27"));
        assert_eq!(invoice["total_amount"], Value::from(123.45));
        assert_eq!(invoice["currency"], Value::from("USD"));
    }

    #[test]
    fn extraction_result_roundtrips_through_json() {
        // A consumer reading our JSON back into `ExtractionResult` (e.g.
        // `invo` itself when chaining a pipeline) must get identical data.
        let original = sample_extraction_result();
        let serialized = serde_json::to_string(&original).expect("serializes");
        let parsed: ExtractionResult = serde_json::from_str(&serialized).expect("roundtrips");

        assert_eq!(parsed.file.hash, original.file.hash);
        assert_eq!(parsed.invoice.vendor, original.invoice.vendor);
        assert_eq!(parsed.invoice.invoice_date, original.invoice.invoice_date);
        assert_eq!(parsed.invoice.total_amount, original.invoice.total_amount);
        assert_eq!(parsed.extraction.provider, original.extraction.provider);
    }
}
