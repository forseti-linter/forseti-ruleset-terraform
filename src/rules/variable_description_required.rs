use forseti_sdk::ruleset::{Rule, RuleContext};
use hcl::Body;
use crate::utils::{HclRule, TerraformUtils};

pub struct VariableDescriptionRequiredRule;

impl Rule for VariableDescriptionRequiredRule {
    fn id(&self) -> &'static str {
        "variable-description-required"
    }

    fn description(&self) -> &'static str {
        "Requires that all Terraform variable blocks include a description for better documentation"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::Value::String("error".to_string())
    }

    fn check(&self, ctx: &mut RuleContext) {
        // Use the HclRule trait's default implementation
        HclRule::check(self, ctx);
    }
}

impl HclRule for VariableDescriptionRequiredRule {
    fn check_hcl(&self, body: &Body, ctx: &mut RuleContext) {
        for block in body.blocks() {
            if block.identifier() == "variable" {
                if let Some(variable_name) = TerraformUtils::get_block_name(block, "variable") {
                    if !TerraformUtils::has_description_attribute(block) {
                        let diagnostic = TerraformUtils::create_missing_description_diagnostic(
                            self.id(),
                            "variable",
                            &variable_name,
                            ctx.text,
                        );
                        ctx.report(diagnostic);
                    }
                }
            }
        }
    }
}
