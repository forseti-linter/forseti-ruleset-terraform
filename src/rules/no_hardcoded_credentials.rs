use forseti_sdk::core::{Diagnostic, LineIndex, Range};
use forseti_sdk::ruleset::{Rule, RuleContext};
use regex::Regex;

pub struct NoHardcodedCredentialsRule;

impl Rule for NoHardcodedCredentialsRule {
    fn id(&self) -> &'static str {
        "no-hardcoded-credentials"
    }

    fn description(&self) -> &'static str {
        "Detects hardcoded credentials such as passwords, API keys, tokens, and secrets in Terraform files"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::Value::String("error".to_string())
    }

    fn check(&self, ctx: &mut RuleContext) {
        let line_index = LineIndex::new(ctx.text);
        
        // Patterns to detect hardcoded credentials
        let patterns = vec![
            (r#"(?i)(password|passwd|pwd)\s*=\s*["'][^"']{1,}["']"#, "Hardcoded password detected"),
            (r#"(?i)(secret|token|key)\s*=\s*["'][^"']{8,}["']"#, "Hardcoded secret/token/key detected"),
            (r#"(?i)(access_key|access-key)\s*=\s*["'][A-Z0-9]{16,}["']"#, "Hardcoded access key detected"),
            (r#"(?i)(private_key|private-key)\s*=\s*["']-----BEGIN"#, "Hardcoded private key detected"),
            (r#"(?i)(api_key|api-key)\s*=\s*["'][A-Za-z0-9]{20,}["']"#, "Hardcoded API key detected"),
        ];

        for (pattern_str, message) in patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                for (line_num, line) in ctx.text.lines().enumerate() {
                    if let Some(mat) = pattern.find(line) {
                        let line_start = ctx.text.lines().take(line_num).map(|l| l.len() + 1).sum::<usize>();
                        let start_pos = line_index.to_pos(line_start + mat.start());
                        let end_pos = line_index.to_pos(line_start + mat.end());

                        let diagnostic = Diagnostic {
                            rule_id: self.id().to_string(),
                            message: message.to_string(),
                            severity: "error".to_string(),
                            range: Range {
                                start: start_pos,
                                end: end_pos,
                            },
                            code: Some("CREDENTIALS".to_string()),
                            suggest: None,
                            docs_url: Some("https://forseti.dev/rules/terraform/no-hardcoded-credentials".to_string()),
                        };

                        ctx.report(diagnostic);
                    }
                }
            }
        }
    }
}
