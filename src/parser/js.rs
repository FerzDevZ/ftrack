use crate::parser::ContentParser;
use crate::types::asset::{Asset, AssetType};
use crate::types::error::Result;
use url::Url;
use regex::Regex;

#[allow(dead_code)]
pub struct JsParser;

impl JsParser {
    #[allow(dead_code)]
    pub fn detect_spa_patterns(&self, content: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        if content.contains("react") || content.contains("_jsx") {
            patterns.push("React".to_string());
        }
        if content.contains("vue") {
            patterns.push("Vue.js".to_string());
        }
        if content.contains("next") {
            patterns.push("Next.js".to_string());
        }
        patterns
    }

    #[allow(dead_code)]
    pub fn discover_api_endpoints(&self, content: &str) -> Vec<String> {
        // Passive API discovery via regex for likely URL patterns in strings
        let re = Regex::new(r#"(https?://[^\s"'<>]+|/api/[^\s"'<>]+)"#).unwrap();
        re.find_iter(content)
            .map(|m| m.as_str().to_string())
            .filter(|s| s.contains("/api/") || s.contains("graphql"))
            .collect()
    }
}

impl ContentParser for JsParser {
    fn extract_links(&self, _content: &str, _base_url: &Url) -> Result<Vec<Url>> {
        Ok(Vec::new()) 
    }

    fn extract_assets(&self, content: &str, base_url: &Url) -> Result<Vec<Asset>> {
        let mut assets = Vec::new();
        // Extract dynamically imported bundles/assets if possible via simple regex
        let re = Regex::new(r#"(?:import|src|href)\s*[:=]\s*["']([^"']+\.(?:js|css|png|jpg|webp))["']"#).unwrap();
        
        for cap in re.captures_iter(content) {
            if let Ok(url) = base_url.join(&cap[1]) {
                assets.push(Asset::new(url, "".into(), AssetType::Other("js-dynamic".to_string()), 0));
            }
        }
        
        Ok(assets)
    }
}
