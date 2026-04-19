use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub confidence: f64,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityMatch {
    pub buyer_name: String,
    pub is_self: bool,
}

// Custom date serialization for YYYY-MM-DD format
mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod option_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_some(&d.format(FORMAT).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) if !s.is_empty() => NaiveDate::parse_from_str(&s, FORMAT)
                .map(Some)
                .map_err(serde::de::Error::custom),
            _ => Ok(None),
        }
    }
}

/// Deserializer that accepts both f64 and string representations of numbers
mod string_or_f64 {
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
            StringOrFloat::String(s) => parse_number_string(&s)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid number: {}", s))),
        }
    }

    fn parse_number_string(s: &str) -> Option<f64> {
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
}

/// Deserializer for Option<f64> that accepts both f64 and string representations
mod option_string_or_f64 {
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
            Null,
        }

        match Option::<StringOrFloat>::deserialize(deserializer)? {
            None => Ok(None),
            Some(StringOrFloat::Null) => Ok(None),
            Some(StringOrFloat::Float(f)) => Ok(Some(f)),
            Some(StringOrFloat::String(s)) if s.is_empty() => Ok(None),
            Some(StringOrFloat::String(s)) => parse_number_string(&s)
                .map(Some)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid number: {}", s))),
        }
    }

    fn parse_number_string(s: &str) -> Option<f64> {
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
}
