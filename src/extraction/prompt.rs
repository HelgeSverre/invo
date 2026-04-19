use crate::config::{Identity, Vendor};

/// Generate the system prompt for invoice extraction
pub fn extraction_system_prompt(identity: &Identity, vendors: &[Vendor]) -> String {
    let mut prompt = String::from(
        r#"You are an invoice/receipt data extraction assistant. Extract structured data from the provided document text.

"#,
    );

    // Add identity context if configured
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

    // Add known vendors if configured
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

/// JSON schema for structured output
pub fn extraction_json_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "extract_invoice",
        "description": "Extract structured data from invoice/receipt text",
        "strict": true,
        "schema": {
            "type": "object",
            "properties": {
                "vendor": {
                    "type": "string",
                    "description": "Company/merchant name that issued the invoice"
                },
                "invoice_date": {
                    "type": "string",
                    "description": "Invoice/transaction date in YYYY-MM-DD format"
                },
                "total_amount": {
                    "type": "number",
                    "description": "Total amount including tax as a JSON number (e.g., 25.00 not \"25.00\")"
                },
                "currency": {
                    "type": "string",
                    "description": "ISO 4217 - 3-letter currency code (USD, EUR, NOK, etc.)"
                },
                "invoice_number": {
                    "type": ["string", "null"],
                    "description": "Invoice/receipt number or reference"
                },
                "due_date": {
                    "type": ["string", "null"],
                    "description": "Payment due date in YYYY-MM-DD format"
                },
                "subtotal": {
                    "type": ["number", "null"],
                    "description": "Amount before tax"
                },
                "tax_amount": {
                    "type": ["number", "null"],
                    "description": "Tax/VAT amount"
                },
                "tax_rate": {
                    "type": ["number", "null"],
                    "description": "Tax rate as decimal (0.25 for 25%)"
                },
                "discount_amount": {
                    "type": ["number", "null"],
                    "description": "Discount amount if any"
                },
                "vendor_website": {
                    "type": ["string", "null"],
                    "description": "Vendor website URL"
                },
                "vendor_email": {
                    "type": ["string", "null"],
                    "description": "Vendor email address"
                },
                "vendor_address": {
                    "type": ["string", "null"],
                    "description": "Vendor physical address"
                },
                "payment_method": {
                    "type": ["string", "null"],
                    "description": "Payment method (Credit Card, Bank Transfer, etc.)"
                },
                "card_last_four": {
                    "type": ["string", "null"],
                    "description": "Last 4 digits of payment card if visible"
                },
                "items": {
                    "type": "array",
                    "description": "Line items on the invoice (empty array if no items visible)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "sku": { "type": ["string", "null"], "description": "Product SKU/code" },
                            "description": { "type": "string", "description": "Item description" },
                            "quantity": { "type": ["number", "null"], "description": "Quantity as a number" },
                            "unit_price": { "type": ["number", "null"], "description": "Price per unit as a number" },
                            "total": { "type": ["number", "null"], "description": "Line total as a number" }
                        },
                        "required": ["description"],
                        "additionalProperties": false
                    }
                },
                "category": {
                    "type": ["string", "null"],
                    "description": "Expense category (Software, Travel, Office, etc.)"
                },
                "description": {
                    "type": ["string", "null"],
                    "description": "Brief description of the purchase or expense (e.g., 'Office supplies for Q1', ´VPS Hosting - subscription´)"
                },
                "buyer_name": {
                    "type": ["string", "null"],
                    "description": "Name of the buyer/customer on the invoice"
                },
                "confidence": {
                    "type": "number",
                    "description": "Confidence score 0.0-1.0"
                }
            },
            "required": ["vendor", "invoice_date", "total_amount", "currency", "items", "confidence"],
            "additionalProperties": false
        }
    })
}

/// Generate the user message with the document text
pub fn extraction_user_message(text: &str) -> String {
    format!(
        "Extract the invoice data from the following document text:\n\n---\n{}\n---",
        text
    )
}
