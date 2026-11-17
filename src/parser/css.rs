use crate::parser::ContentParser;
use crate::types::asset::{Asset, AssetType};
use crate::types::error::Result;
use url::Url;
use lightningcss::stylesheet::{StyleSheet, ParserOptions};
use lightningcss::visitor::{Visitor, Visit, VisitTypes};
use lightningcss::values::url::Url as CssUrl;

#[allow(dead_code)]
pub struct CssParser;

impl ContentParser for CssParser {
    fn extract_links(&self, _content: &str, _base_url: &Url) -> Result<Vec<Url>> {
        Ok(Vec::new()) // CSS doesn't usually contain navigateable links
    }

    fn extract_assets(&self, content: &str, base_url: &Url) -> Result<Vec<Asset>> {
        let mut visitor = CssUrlVisitor {
            base_url: base_url.clone(),
            assets: Vec::new(),
        };

        if let Ok(mut stylesheet) = StyleSheet::parse(content, ParserOptions::default()) {
            let _ = stylesheet.visit(&mut visitor);
        }

        Ok(visitor.assets)
    }
}

#[allow(dead_code)]
struct CssUrlVisitor {
    base_url: Url,
    assets: Vec<Asset>
}

impl<'i> Visitor<'i> for CssUrlVisitor {
    type Error = std::convert::Infallible;

    fn visit_types(&self) -> VisitTypes {
        VisitTypes::URLS
    }

    fn visit_url(&mut self, url: &mut CssUrl<'i>) -> std::result::Result<(), <Self as Visitor<'i>>::Error> {
        if let Ok(full_url) = self.base_url.join(&url.url) {
            self.assets.push(Asset::new(full_url, "".into(), AssetType::Other("css-asset".to_string()), 0));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://example.com/css/site.css").expect("base URL parses")
    }

    #[test]
    fn url_references_are_resolved_against_the_stylesheet() {
        let css = r#"
            @font-face { font-family: Inter; src: url("../fonts/inter.woff2"); }
            .hero { background-image: url("/img/hero.jpg"); }
        "#;

        let assets = CssParser.extract_assets(css, &base()).expect("css parses");

        let urls: Vec<&str> = assets.iter().map(|a| a.url.as_str()).collect();
        assert!(urls.contains(&"https://example.com/fonts/inter.woff2"));
        assert!(urls.contains(&"https://example.com/img/hero.jpg"));
        assert!(assets.iter().all(|a| matches!(a.asset_type, AssetType::Other(ref kind) if kind == "css-asset")));
    }

    #[test]
    fn a_stylesheet_contributes_no_navigation_links() {
        let links = CssParser.extract_links(".a { color: red }", &base()).expect("css parses");

        assert!(links.is_empty());
    }

    #[test]
    fn broken_css_does_not_abort_the_crawl() {
        let assets = CssParser.extract_assets("{{ not css", &base()).expect("no error is returned");

        assert!(assets.is_empty());
    }
}
