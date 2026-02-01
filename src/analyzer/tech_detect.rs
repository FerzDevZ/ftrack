use crate::analyzer::TechDetector;
use crate::types::asset::Asset;
use std::collections::HashSet;

pub struct DefaultTechDetector;

impl TechDetector for DefaultTechDetector {
    fn detect(&self, _asset: &Asset, content: &str) -> Vec<String> {
        let mut tech = HashSet::new();

        let signatures = [
            ("react", "React"),
            ("vue", "Vue.js"),
            ("angular", "Angular"),
            ("jquery", "jQuery"),
            ("bootstrap", "Bootstrap"),
            ("tailwind", "Tailwind CSS"),
            ("next.js", "Next.js"),
            ("nuxt", "Nuxt.js"),
            ("svelte", "Svelte"),
            ("wordpress", "WordPress"),
        ];

        let content_lower = content.to_lowercase();
        for (sig, name) in signatures {
            if content_lower.contains(sig) {
                tech.insert(name.to_string());
            }
        }

        tech.into_iter().collect()
    }
}
