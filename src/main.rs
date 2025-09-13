use anyhow::Result;
use forseti_sdk::core::{FileContext, PreprocessingContext, RulesetCapabilities};
use forseti_sdk::ruleset::{Ruleset, RulesetOptions, RulesetServer};
use serde_json::{json};
use std::collections::HashMap;

mod rules;
mod utils;

use rules::*;

struct TerraformRuleset;

impl RulesetOptions for TerraformRuleset {

    fn create_ruleset(&self) -> Ruleset {
        create_terraform_ruleset()
    }

    fn get_capabilities(&self) -> RulesetCapabilities {
        RulesetCapabilities {
            ruleset_id: "terraform".to_string(),
            version: "0.1.0".to_string(),
            file_patterns: vec!["*.tf".to_string(), "*.tfvars".to_string()],
            max_file_size: Some(5 * 1024 * 1024), // 5MB limit for Terraform files
            annotation_prefixes: vec![
                "#".to_string(),  // HCL/Terraform single-line comments
                "//".to_string(), // Alternative comment syntax sometimes used
            ],
            rules: vec![], // Will be populated by the server
            default_config: self.get_default_config(),
            config_settings: vec![], // No custom settings for now, rule enable/disable will be auto-injected
        }
    }

    fn preprocess_files(&self, file_uris: &[String]) -> Result<PreprocessingContext> {
        let mut files = Vec::new();
        let mut global_context = HashMap::new();

        // Terraform engine: gather Terraform-specific metadata
        let mut tf_files = 0;
        let mut tfvars_files = 0;

        for uri in file_uris {
            let mut context = HashMap::new();

            // Only gather lightweight file metadata
            if uri.starts_with("file://") {
                let path = uri.strip_prefix("file://").unwrap_or(uri);
                if let Ok(metadata) = std::fs::metadata(path) {
                    context.insert("file_size".to_string(), json!(metadata.len()));
                    context.insert("is_file".to_string(), json!(metadata.is_file()));
                }

                // Categorize Terraform files
                let path_obj = std::path::Path::new(path);
                if let Some(ext) = path_obj.extension() {
                    let extension = ext.to_string_lossy();
                    context.insert("extension".to_string(), json!(extension));

                    match extension.as_ref() {
                        "tf" => {
                            tf_files += 1;
                            context
                                .insert("terraform_file_type".to_string(), json!("configuration"));
                        }
                        "tfvars" => {
                            tfvars_files += 1;
                            context.insert("terraform_file_type".to_string(), json!("variables"));
                        }
                        _ => {}
                    }
                }

                // Check if it's in a .terraform directory (should be ignored)
                if path.contains("/.terraform/") {
                    context.insert("terraform_generated".to_string(), json!(true));
                }
            }

            files.push(FileContext {
                uri: uri.clone(),
                content: String::new(), // Empty - rulesets will load content themselves
                language: infer_language(uri),
                context,
            });
        }

        // Global context with Terraform-specific stats
        global_context.insert("total_files".to_string(), json!(files.len()));
        global_context.insert("tf_files".to_string(), json!(tf_files));
        global_context.insert("tfvars_files".to_string(), json!(tfvars_files));
        global_context.insert("ruleset_type".to_string(), json!("terraform"));

        Ok(PreprocessingContext {
            ruleset_id: "terraform".to_string(),
            files,
            global_context,
        })
    }
}

fn create_terraform_ruleset() -> Ruleset {
    Ruleset::new("terraform")
        .with_rule(Box::new(NoHardcodedCredentialsRule))
        .with_rule(Box::new(RequireProviderVersionRule))
        .with_rule(Box::new(NoDeprecatedInterpolationRule))
        .with_rule(Box::new(ResourceNamingConventionRule))
        .with_rule(Box::new(VariableDescriptionRequiredRule))
        .with_rule(Box::new(OutputDescriptionRequiredRule))
}

fn infer_language(uri: &str) -> Option<String> {
    let path = if uri.starts_with("file://") {
        uri.strip_prefix("file://").unwrap_or(uri)
    } else {
        uri
    };

    match std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
    {
        Some("tf") => Some("terraform".to_string()),
        Some("tfvars") => Some("terraform-vars".to_string()),
        _ => None,
    }
}

fn main() -> Result<()> {
    let mut server = RulesetServer::new(Box::new(TerraformRuleset));
    server.run_stdio()
}
