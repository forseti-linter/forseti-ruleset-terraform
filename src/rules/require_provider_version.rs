use forseti_sdk::ruleset::{Rule, RuleContext};
use hcl::{Body, Block};
use crate::utils::{HclRule, TerraformUtils};

pub struct RequireProviderVersionRule;

impl Rule for RequireProviderVersionRule {
    fn id(&self) -> &'static str {
        "require-provider-version"
    }

    fn description(&self) -> &'static str {
        "Ensures Terraform providers specify version constraints for better dependency management and reproducibility"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::Value::String("error".to_string())
    }

    fn check(&self, ctx: &mut RuleContext) {
        // Use the HclRule trait's default implementation
        HclRule::check(self, ctx);
    }
}

impl HclRule for RequireProviderVersionRule {
    fn check_hcl(&self, body: &Body, ctx: &mut RuleContext) {
        // Look for terraform blocks and check required_providers
        for block in body.blocks() {
            if block.identifier() == "terraform" {
                self.check_required_providers_block(block, ctx);
            }
        }
    }
}

impl RequireProviderVersionRule {
    fn check_required_providers_block(&self, terraform_block: &Block, ctx: &mut RuleContext) {
        // Look for required_providers block within terraform block
        for nested_block in terraform_block.body().blocks() {
            if nested_block.identifier() == "required_providers" {
                self.check_provider_entries(nested_block, ctx);
            }
        }
    }

    fn check_provider_entries(&self, required_providers_block: &Block, ctx: &mut RuleContext) {
        // Check each attribute in the required_providers block
        for attr in required_providers_block.body().attributes() {
            let provider_name = attr.key();
            
            // Check if this provider config has a version attribute
            let has_version = match attr.expr() {
                hcl::Expression::Object(obj) => {
                    obj.iter().any(|(key, _)| {
                        TerraformUtils::debug_to_string(key) == "version"
                    })
                }
                _ => false, // Provider is not an object, so no version specified
            };

            if !has_version {
                let diagnostic = TerraformUtils::create_provider_version_diagnostic(
                    provider_name,
                    ctx.text,
                );
                ctx.report(diagnostic);
            }
        }
    }
}
