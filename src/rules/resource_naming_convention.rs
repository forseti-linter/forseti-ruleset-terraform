use forseti_sdk::ruleset::{Rule, RuleContext};
use hcl::Body;
use regex::Regex;
use crate::utils::{HclRule, TerraformUtils};

pub struct ResourceNamingConventionRule;

impl Rule for ResourceNamingConventionRule {
    fn id(&self) -> &'static str {
        "resource-naming-convention"
    }

    fn description(&self) -> &'static str {
        "Enforces snake_case naming convention for Terraform resources to maintain consistency"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::Value::String("error".to_string())
    }

    fn check(&self, ctx: &mut RuleContext) {
        // Use the HclRule trait's default implementation
        HclRule::check(self, ctx);
    }
}

impl HclRule for ResourceNamingConventionRule {
    fn check_hcl(&self, body: &Body, ctx: &mut RuleContext) {
        // Valid naming pattern: snake_case starting with letter
        let valid_name_pattern = Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();

        for block in body.blocks() {
            let block_type = block.identifier();
            
            // Check naming for these block types
            if matches!(block_type, "resource" | "data" | "variable" | "output" | "locals") {
                if let Some(name) = TerraformUtils::get_block_name(block, block_type) {
                    if !valid_name_pattern.is_match(&name) {
                        let diagnostic = TerraformUtils::create_naming_convention_diagnostic(
                            block_type,
                            &name,
                            ctx.text,
                        );
                        ctx.report(diagnostic);
                    }
                }
            }
        }
    }
}
