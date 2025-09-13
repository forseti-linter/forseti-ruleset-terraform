use forseti_sdk::core::{Diagnostic, Position, Range};
use forseti_sdk::ruleset::RuleContext;
use hcl::{Block, BlockLabel, Body};

/// Shared utilities for Terraform engine rules
pub struct TerraformUtils;

impl TerraformUtils {
    /// Parse HCL content and return Body, or None if parsing fails
    pub fn parse_hcl(text: &str) -> Option<Body> {
        hcl::from_str::<Body>(text).ok()
    }

    /// Convert byte offset to LSP Position
    pub fn offset_to_position(offset: usize, text: &str) -> Position {
        let mut line = 0;
        let mut character = 0;
        let mut current_offset = 0;

        for ch in text.chars() {
            if current_offset >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }

            current_offset += ch.len_utf8();
        }

        Position { line, character }
    }

    /// Extract the string value from a BlockLabel
    pub fn block_label_to_string(label: &BlockLabel) -> String {
        format!("{:?}", label).trim_matches('"').to_string()
    }

    /// Extract the string value from any Debug-formattable type (for ObjectKey, etc.)
    pub fn debug_to_string<T: std::fmt::Debug>(item: &T) -> String {
        format!("{:?}", item).trim_matches('"').to_string()
    }

    /// Get the name of a block (first label for most blocks, second for resource/data)
    pub fn get_block_name(block: &Block, block_type: &str) -> Option<String> {
        match block_type {
            "resource" | "data" => {
                // resource "aws_instance" "my_instance" - get "my_instance"
                block.labels().get(1).map(Self::block_label_to_string)
            }
            "variable" | "output" | "locals" => {
                // variable "my_var" - get "my_var"
                block.labels().first().map(Self::block_label_to_string)
            }
            _ => None,
        }
    }

    /// Create a diagnostic for a missing description attribute
    pub fn create_missing_description_diagnostic(
        rule_id: &str,
        block_type: &str,
        block_name: &str,
        text: &str,
    ) -> Diagnostic {
        let block_text = format!("{} \"{}\"", block_type, block_name);
        let block_start = text.find(&block_text).unwrap_or(0);

        let start_pos = Self::offset_to_position(block_start, text);
        let end_pos = Self::offset_to_position(block_start + block_text.len(), text);

        Diagnostic {
            rule_id: rule_id.to_string(),
            message: format!(
                "{} '{}' should have a description",
                Self::capitalize_first(block_type),
                block_name
            ),
            severity: "warn".to_string(),
            range: Range {
                start: start_pos,
                end: end_pos,
            },
            code: Some("MISSING_DESCRIPTION".to_string()),
            suggest: None,
            docs_url: Some(format!("https://forseti.dev/rules/terraform/{}", rule_id)),
        }
    }

    /// Create a diagnostic for naming convention violations
    pub fn create_naming_convention_diagnostic(
        block_type: &str,
        block_name: &str,
        text: &str,
    ) -> Diagnostic {
        // Try to find the specific name position for better highlighting
        let name_pattern = format!("\"{}\"", block_name);
        let name_start = if let Some(pos) = text.find(&name_pattern) {
            pos + 1 // Skip opening quote
        } else {
            text.find(block_name).unwrap_or(0)
        };

        let start_pos = Self::offset_to_position(name_start, text);
        let end_pos = Self::offset_to_position(name_start + block_name.len(), text);

        Diagnostic {
            rule_id: "resource-naming-convention".to_string(),
            message: format!(
                "{} name '{}' should follow snake_case convention",
                block_type, block_name
            ),
            severity: "warn".to_string(),
            range: Range {
                start: start_pos,
                end: end_pos,
            },
            code: Some("NAMING_CONVENTION".to_string()),
            suggest: None,
            docs_url: Some(
                "https://forseti.dev/rules/terraform/resource-naming-convention".to_string(),
            ),
        }
    }

    /// Create a diagnostic for missing provider version
    pub fn create_provider_version_diagnostic(provider_name: &str, text: &str) -> Diagnostic {
        let provider_text = format!("{} =", provider_name);
        let provider_start = text.find(&provider_text).unwrap_or(0);

        let start_pos = Self::offset_to_position(provider_start, text);
        let end_pos = Self::offset_to_position(provider_start + provider_name.len(), text);

        Diagnostic {
            rule_id: "require-provider-version".to_string(),
            message: format!(
                "Provider '{}' should specify a version constraint",
                provider_name
            ),
            severity: "warn".to_string(),
            range: Range {
                start: start_pos,
                end: end_pos,
            },
            code: Some("PROVIDER_VERSION".to_string()),
            suggest: None,
            docs_url: Some(
                "https://forseti.dev/rules/terraform/require-provider-version".to_string(),
            ),
        }
    }

    /// Check if a block has a description attribute
    pub fn has_description_attribute(block: &Block) -> bool {
        block
            .body()
            .attributes()
            .any(|attr| attr.key() == "description")
    }

    /// Capitalize first letter of a string
    fn capitalize_first(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}

/// Trait for rules that need common HCL parsing functionality
pub trait HclRule {
    /// Check rule with HCL parsing handled automatically
    fn check_hcl(&self, body: &Body, ctx: &mut RuleContext);

    /// Default implementation that handles HCL parsing
    fn check(&self, ctx: &mut RuleContext) {
        if let Some(body) = TerraformUtils::parse_hcl(ctx.text) {
            self.check_hcl(&body, ctx);
        }
        // If parsing fails, silently skip (file might be invalid HCL)
    }
}
