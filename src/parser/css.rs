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
