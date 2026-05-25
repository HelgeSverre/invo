use crate::config::{Identity, Vendor};

const BASE_PROMPT: &str = r#"You are an invoice/receipt data extraction assistant. Extract structured data from the provided document text.

"#;

const EXTRACTION_RULES: &str = r#"EXTRACTION RULES:
1. Dates MUST be in YYYY-MM-DD format (e.g., 2025-01-15)
2. Currency MUST be ISO 4217 3-letter code (USD, EUR, NOK, GBP, SEK, DKK, etc.)
3. ALL numeric amounts MUST be JSON numbers, NOT strings (e.g., 25.00 not "25.00")
4. Do NOT include currency symbols in amounts - just the number
5. Extract ALL line items if visible
6. Set confidence based on text clarity (0.0-1.0)
7. If unsure about a field, set it to null rather than guess
8. The vendor is the company that issued the invoice, NOT the buyer

Extract the invoice data and respond with valid JSON matching the schema."#;

pub fn extraction_system_prompt(identity: &Identity, vendors: &[Vendor]) -> String {
    let mut prompt = String::from(BASE_PROMPT);

    push_identity_context(&mut prompt, identity);
    push_vendor_context(&mut prompt, vendors);
    prompt.push_str(EXTRACTION_RULES);

    prompt
}

fn push_identity_context(prompt: &mut String, identity: &Identity) {
    if !identity.is_configured() {
        return;
    }

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
    if let Some(address) = &identity.address {
        prompt.push_str(&format!("- Address: {}\n", address));
    }
    if let Some(vat_number) = &identity.vat_number {
        prompt.push_str(&format!("- VAT number: {}\n", vat_number));
    }
    if !identity.aliases.is_empty() {
        prompt.push_str(&format!("- Aliases: {}\n", identity.aliases.join(", ")));
    }
    prompt.push_str(
        "\nIf you see any of these names, addresses, VAT numbers, or aliases on the invoice, they are the BUYER, not the vendor.\n\n",
    );
}

fn push_vendor_context(prompt: &mut String, vendors: &[Vendor]) {
    if vendors.is_empty() {
        return;
    }

    prompt.push_str("KNOWN VENDORS:\n");
    for vendor in vendors {
        prompt.push_str(&format!("- {}", vendor.name));
        if !vendor.patterns.is_empty() {
            prompt.push_str(&format!(" (patterns: {})", vendor.patterns.join(", ")));
        }
        if let Some(category) = &vendor.category {
            prompt.push_str(&format!(" [category: {}]", category));
        }
        if let Some(currency) = &vendor.currency {
            prompt.push_str(&format!(" [expected currency: {}]", currency));
        }
        if let Some(hints) = &vendor.hints {
            prompt.push_str(&format!(" [hints: {}]", hints));
        }
        prompt.push('\n');
    }
    prompt.push('\n');
}

pub fn extraction_user_message(text: &str) -> String {
    format!(
        "Extract the invoice data from the following document text:\n\n---\n{}\n---",
        text
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_prompt_includes_address_and_vat_number() {
        let identity = Identity {
            company: Some("Liseth Solutions AS".to_string()),
            address: Some("Examplegata 1, Oslo".to_string()),
            vat_number: Some("NO123456789MVA".to_string()),
            ..Identity::default()
        };

        let prompt = extraction_system_prompt(&identity, &[]);

        assert!(prompt.contains("- Company: Liseth Solutions AS"));
        assert!(prompt.contains("- Address: Examplegata 1, Oslo"));
        assert!(prompt.contains("- VAT number: NO123456789MVA"));
        assert!(prompt.contains("BUYER, not the vendor"));
    }

    #[test]
    fn vendor_prompt_includes_currency_and_hints() {
        let vendors = vec![Vendor {
            name: "Anthropic".to_string(),
            patterns: vec!["Anthropic, PBC".to_string()],
            currency: Some("USD".to_string()),
            category: Some("software".to_string()),
            hints: Some("Legal entity appears as Anthropic, PBC".to_string()),
        }];

        let prompt = extraction_system_prompt(&Identity::default(), &vendors);

        assert!(prompt.contains("- Anthropic"));
        assert!(prompt.contains("[expected currency: USD]"));
        assert!(prompt.contains("[hints: Legal entity appears as Anthropic, PBC]"));
    }
}
