use super::schema::Invoice;
use crate::config::{Identity, Vendor};
use std::sync::OnceLock;

pub fn extraction_system_prompt(identity: &Identity, vendors: &[Vendor]) -> String {
    let mut prompt = String::from(
        r#"You are an invoice/receipt data extraction assistant. Extract structured data from the provided document text.

"#,
    );

    if identity.is_configured() {
        prompt.push_str("IDENTITY CONTEXT (the user/buyer):\n");
        if let Some(company) = &identity.company {
            prompt.push_str(&format!("- Company: {}\n", company));
        }
        if let Some(name) = &identity.name {
            prompt.push_str(&format!("- Name: {}\n", name));
        }
        if let Some(email) = &identity.email {
            prompt.push_str(&format!("- Email: {}\n", email));
        }
        if !identity.aliases.is_empty() {
            prompt.push_str(&format!("- Aliases: {}\n", identity.aliases.join(", ")));
        }
        prompt.push_str("\nIf you see any of these names on the invoice, they are the BUYER, not the vendor.\n\n");
    }

    if !vendors.is_empty() {
        prompt.push_str("KNOWN VENDORS:\n");
        for vendor in vendors {
            prompt.push_str(&format!("- {}", vendor.name));
            if !vendor.patterns.is_empty() {
                prompt.push_str(&format!(" (patterns: {})", vendor.patterns.join(", ")));
            }
            if let Some(category) = &vendor.category {
                prompt.push_str(&format!(" [category: {}]", category));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    prompt.push_str(
        r#"EXTRACTION RULES:
1. Dates MUST be in YYYY-MM-DD format (e.g., 2025-01-15)
2. Currency MUST be ISO 4217 3-letter code (USD, EUR, NOK, GBP, SEK, DKK, etc.)
3. ALL numeric amounts MUST be JSON numbers, NOT strings (e.g., 25.00 not "25.00")
4. Do NOT include currency symbols in amounts - just the number
5. Extract ALL line items if visible
6. Set confidence based on text clarity (0.0-1.0)
7. If unsure about a field, set it to null rather than guess
8. The vendor is the company that issued the invoice, NOT the buyer

Extract the invoice data and respond with valid JSON matching the schema."#,
    );

    prompt
}

pub fn invoice_schema() -> &'static serde_json::Value {
    static SCHEMA: OnceLock<serde_json::Value> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::to_value(schemars::schema_for!(Invoice))
            .expect("Invoice schema must be serializable")
    })
}

pub fn extraction_user_message(text: &str) -> String {
    format!(
        "Extract the invoice data from the following document text:\n\n---\n{}\n---",
        text
    )
}
