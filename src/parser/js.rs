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
        /* Bundles spell the framework the way its source does — `React`,
           `Vue` — so matching is case-insensitive. */
        let haystack = content.to_lowercase();
        let mut patterns = Vec::new();
        if haystack.contains("react") || haystack.contains("_jsx") {
            patterns.push("React".to_string());
        }
        if haystack.contains("vue") {
            patterns.push("Vue.js".to_string());
        }
        if haystack.contains("next") {
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
        /* Covers `import("./chunk.js")` as well as `src="x.js"` and `href: "x.css"`. */
        let re = Regex::new(r#"(?:import|src|href)\s*(?:[:=(]\s*)["']([^"']+\.(?:js|css|png|jpg|webp))["']"#).unwrap();
        
        for cap in re.captures_iter(content) {
            if let Ok(url) = base_url.join(&cap[1]) {
                assets.push(Asset::new(url, "".into(), AssetType::Other("js-dynamic".to_string()), 0));
            }
        }
        
        Ok(assets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spa_markers_name_the_framework() {
        let parser = JsParser;

        assert_eq!(parser.detect_spa_patterns("var x = React.createElement"), vec!["React"]);
        assert_eq!(parser.detect_spa_patterns("window.__NEXT_DATA__"), vec!["Next.js"]);
        assert_eq!(parser.detect_spa_patterns("new Vue({ el: '#app' })"), vec!["Vue.js"]);
        assert_eq!(parser.detect_spa_patterns("__NEXT_DATA__"), vec!["Next.js"]);
        assert!(parser.detect_spa_patterns("console.log('plain')").is_empty());
    }

    #[test]
    fn api_discovery_keeps_only_endpoint_like_urls() {
        let parser = JsParser;
        let bundle = r#"
            fetch("/api/orders");
            const client = createClient("https://api.example.com/graphql");
            location.href = "https://example.com/pricing";
        "#;

        let endpoints = parser.discover_api_endpoints(bundle);

        assert_eq!(
            endpoints,
            vec!["/api/orders", "https://api.example.com/graphql"]
        );
    }

    #[test]
    fn dynamically_imported_assets_are_found() {
        let parser = JsParser;
        let bundle = r#"
            import("./chunk.abc123.js");
            const img = document.createElement("img"); img.src = "hero.webp";
            const link = { href: "theme.css" };
        "#;

        let assets = parser.extract_assets(bundle, &Url::parse("https://example.com/app.js").unwrap())
            .expect("assets parse");

        assert_eq!(assets.len(), 3);
        assert_eq!(assets[0].url.as_str(), "https://example.com/chunk.abc123.js");
        assert_eq!(assets[1].url.as_str(), "https://example.com/hero.webp");
        assert_eq!(assets[2].url.as_str(), "https://example.com/theme.css");
        assert!(assets.iter().all(|a| matches!(a.asset_type, AssetType::Other(_))));
    }

    #[test]
    fn a_javascript_file_contributes_no_navigation_links() {
        let parser = JsParser;
        let links = parser
            .extract_links("fetch('/api/orders')", &Url::parse("https://example.com/app.js").unwrap())
            .expect("links parse");

        assert!(links.is_empty());
    }
}
