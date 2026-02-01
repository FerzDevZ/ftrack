use crate::analyzer::SecretScanner;
use crate::types::leak::Leak;
use regex::Regex;

pub struct DefaultSecretScanner {
    rules: Vec<SecretRule>,
}

struct SecretRule {
    kind: String,
    regex: Regex,
}

impl DefaultSecretScanner {
    pub fn new() -> Self {
        let rules = vec![
            SecretRule {
                kind: "Google API Key".to_string(),
                regex: Regex::new(r"AIza[0-9A-Za-z-_]{35}").unwrap(),
            },
            SecretRule {
                kind: "AWS Access Key".to_string(),
                regex: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            },
            SecretRule {
                kind: "Stripe Secret Key".to_string(),
                regex: Regex::new(r"sk_live_[0-9a-zA-Z]{24}").unwrap(),
            },
            SecretRule {
                kind: "Discord Bot Token".to_string(),
                regex: Regex::new(r"[MN][A-Za-z\d]{23}\.[\w-]{6}\.[\w-]{27}").unwrap(),
            },
            SecretRule {
                kind: "Slack Webhook URL".to_string(),
                regex: Regex::new(r"https://hooks\.slack\.com/services/T[A-Z0-9]{8}/B[A-Z0-9]{8}/[A-Za-z0-9]{24}").unwrap(),
            },
            SecretRule {
                kind: "Generic Password/Secret".to_string(),
                regex: Regex::new(r#"(?i)(password|secret|key|auth|token|api_key|access_token|client_secret)\s*[:=]\s*["']([^"']+)["']"#).unwrap(),
            },
        ];
        Self { rules }
    }
}

impl SecretScanner for DefaultSecretScanner {
    fn scan(&self, url: &str, content: &str) -> Vec<Leak> {
        let mut leaks = Vec::new();
        for rule in &self.rules {
            for mat in rule.regex.find_iter(content) {
                let start = mat.start().saturating_sub(30);
                let end = (mat.end() + 30).min(content.len());
                let context = content[start..end].replace('\n', " ").trim().to_string();

                leaks.push(Leak {
                    kind: rule.kind.clone(),
                    value: mat.as_str().to_string(),
                    url: url.to_string(),
                    context,
                });
            }
        }
        leaks
    }
}
