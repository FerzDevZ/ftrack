use crate::resolver::ContentRewriter;
use crate::types::error::Result;
use url::Url;
use std::collections::HashMap;
use std::path::PathBuf;
use regex::{Regex, Captures};

pub struct DefaultContentRewriter;

impl ContentRewriter for DefaultContentRewriter {
    fn rewrite(&self, content: &str, mapping: &HashMap<Url, PathBuf>) -> Result<String> {
        let mut rewritten = content.to_string();

        // Simple regex rewrite for various attributes (src, href, url())
        // In a production-grade tool, we might use a proper HTML/CSS walker for 100% fidelity,
        // but regex is often faster and covers 95% of cases if crafted carefully.
        
        let patterns = [
            r#"(src|href)\s*[:=]\s*["']([^"']+)["']"#,
            r#"url\(['"]?([^'"\)]+)['"]?\)"#,
        ];

        for pattern in patterns {
            let re = Regex::new(pattern).unwrap();
            rewritten = re.replace_all(&rewritten, |caps: &Captures| {
                let original_val = &caps[caps.len() - 1];
                // Try to find if this URL is in our mapping
                // Note: This requires absolute URLs in mapping, or normalization here.
                // For simplicity in the skeleton, we look for exact matches or ends_with.
                let mut replacement = original_val.to_string();
                for (url, local_path) in mapping {
                    if original_val.contains(url.path()) {
                        replacement = local_path.to_string_lossy().to_string();
                        break;
                    }
                }

                if caps.len() == 3 {
                    format!("{}=\"{}\"", &caps[1], replacement)
                } else {
                    format!("url(\"{}\")", replacement)
                }
            }).to_string();
        }

        Ok(rewritten)
    }
}
