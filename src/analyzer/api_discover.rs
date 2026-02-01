use crate::analyzer::ApiDiscovery;
use crate::types::endpoint::Endpoint;
use regex::Regex;

pub struct DefaultApiDiscoverer {
    api_patterns: Vec<ApiPattern>,
}

struct ApiPattern {
    #[allow(dead_code)]
    label: String,
    regex: Regex,
}

impl DefaultApiDiscoverer {
    pub fn new() -> Self {
        let api_patterns = vec![
            ApiPattern {
                label: "RESTful Path".to_string(),
                regex: Regex::new(r#"(?:https?://[^\s"'<>]+)?/api/v[0-9]+/[^\s"'<>]+"#).unwrap(),
            },
            ApiPattern {
                label: "GraphQL".to_string(),
                regex: Regex::new(r#"(?i)/graphql"#).unwrap(),
            },
            ApiPattern {
                label: "JSON-RPC/Web".to_string(),
                regex: Regex::new(r#"(?i)/rpc|/v[0-9]+/json"#).unwrap(),
            },
            ApiPattern {
                label: "Firebase/GCP".to_string(),
                regex: Regex::new(r#"(?i)firebase\.googleapis\.com/.+"#).unwrap(),
            },
            ApiPattern {
                label: "Generic API Path".to_string(),
                regex: Regex::new(r#"/api/[0-9a-zA-Z\-_/]+"#).unwrap(),
            },
        ];
        Self { api_patterns }
    }
}

impl ApiDiscovery for DefaultApiDiscoverer {
    fn discover(&self, source_url: &str, content: &str) -> Vec<Endpoint> {
        let mut endpoints = Vec::new();
        // 1. Regex based discovery
        for pattern in &self.api_patterns {
            for mat in pattern.regex.find_iter(content) {
                let found = mat.as_str().to_string();
                
                // Context for reference
                let start = mat.start().saturating_sub(30);
                let end = (mat.end() + 30).min(content.len());
                let context = content[start..end].replace('\n', " ").trim().to_string();

                endpoints.push(Endpoint {
                    method: "DISCOVERED".to_string(), // We don't know the exact verb yet
                    url: found,
                    source_url: source_url.to_string(),
                    context,
                });
            }
        }

        // Deduplicate locally
        endpoints.dedup_by(|a, b| a.url == b.url);
        endpoints
    }
}
