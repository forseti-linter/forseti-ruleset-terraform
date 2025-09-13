use forseti_sdk::core::{Diagnostic, LineIndex, Range};
use forseti_sdk::ruleset::{Rule, RuleContext};
use regex::Regex;

pub struct NoDeprecatedInterpolationRule;

impl Rule for NoDeprecatedInterpolationRule {
    fn id(&self) -> &'static str {
        "no-deprecated-interpolation"
    }

    fn description(&self) -> &'static str {
        "Flags deprecated interpolation syntax in Terraform (e.g., \"${var.name}\") in favor of modern expressions"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::Value::String("error".to_string())
    }

    fn check(&self, ctx: &mut RuleContext) {
        let line_index = LineIndex::new(ctx.text);
        
        // Look for deprecated interpolation syntax like "${var.name}" in strings
        // that should just be var.name in modern Terraform
        let deprecated_pattern = Regex::new(r#""[^"]*\$\{([^}]+)\}[^"]*""#).unwrap();

        for (line_num, line) in ctx.text.lines().enumerate() {
            for mat in deprecated_pattern.find_iter(line) {
                // Skip if this is a complex interpolation that actually needs ${}
                let interpolation_content = &line[mat.start()..mat.end()];
                if interpolation_content.contains(" ") || 
                   interpolation_content.contains("+") || 
                   interpolation_content.contains("*") ||
                   interpolation_content.contains("/") ||
                   interpolation_content.contains("(") {
                    continue; // This is a complex expression that needs interpolation
                }

                let line_start = ctx.text.lines().take(line_num).map(|l| l.len() + 1).sum::<usize>();
                let start_pos = line_index.to_pos(line_start + mat.start());
                let end_pos = line_index.to_pos(line_start + mat.end());

                let diagnostic = Diagnostic {
                    rule_id: self.id().to_string(),
                    message: "Deprecated interpolation syntax found. Use direct variable reference instead".to_string(),
                    severity: "warn".to_string(),
                    range: Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    code: Some("DEPRECATED_INTERPOLATION".to_string()),
                    suggest: None,
                    docs_url: Some("https://forseti.dev/rules/terraform/no-deprecated-interpolation".to_string()),
                };

                ctx.report(diagnostic);
            }
        }
    }
}
