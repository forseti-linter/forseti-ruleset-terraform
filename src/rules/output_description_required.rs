use forseti_sdk::ruleset::{Rule, RuleContext};
use hcl::Body;
use crate::utils::{HclRule, TerraformUtils};

pub struct OutputDescriptionRequiredRule;

impl Rule for OutputDescriptionRequiredRule {
    fn id(&self) -> &'static str {
        "output-description-required"
    }

    fn description(&self) -> &'static str {
        "Requires that all Terraform output blocks include a description for better documentation"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::Value::String("error".to_string())
    }

    fn check(&self, ctx: &mut RuleContext) {
        // Use the HclRule trait's default implementation
        HclRule::check(self, ctx);
    }
}

impl HclRule for OutputDescriptionRequiredRule {
    fn check_hcl(&self, body: &Body, ctx: &mut RuleContext) {
        for block in body.blocks() {
            if block.identifier() == "output" {
                if let Some(output_name) = TerraformUtils::get_block_name(block, "output") {
                    if !TerraformUtils::has_description_attribute(block) {
                        let diagnostic = TerraformUtils::create_missing_description_diagnostic(
                            self.id(),
                            "output",
                            &output_name,
                            ctx.text,
                        );
                        ctx.report(diagnostic);
                    }
                }
            }
        }
    }
}
