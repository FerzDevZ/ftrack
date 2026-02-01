use crate::parser::ContentParser;
use crate::types::asset::{Asset, AssetType};
use crate::types::error::Result;
use scraper::{Html, Selector};
use url::Url;

pub struct HtmlParser;

impl ContentParser for HtmlParser {
    fn extract_links(&self, content: &str, base_url: &Url) -> Result<Vec<Url>> {
        let document = Html::parse_document(content);
        let selector = Selector::parse("a[href]").unwrap();
        
        let mut links = Vec::new();
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(url) = base_url.join(href) {
                    links.push(url);
                }
            }
        }
        Ok(links)
    }

    fn extract_assets(&self, content: &str, base_url: &Url) -> Result<Vec<Asset>> {
        let document = Html::parse_document(content);
        let mut assets = Vec::new();

        // 1. Images
        let img_selector = Selector::parse("img[src]").unwrap();
        for element in document.select(&img_selector) {
            if let Some(src) = element.value().attr("src") {
                if let Ok(url) = base_url.join(src) {
                    assets.push(Asset::new(url, "".into(), AssetType::Image, 0));
                }
            }
        }

        // 2. Scripts
        let script_selector = Selector::parse("script[src]").unwrap();
        for element in document.select(&script_selector) {
            if let Some(src) = element.value().attr("src") {
                if let Ok(url) = base_url.join(src) {
                    assets.push(Asset::new(url, "".into(), AssetType::Js, 0));
                }
            }
        }

        // 3. Stylesheets
        let link_selector = Selector::parse("link[rel='stylesheet']").unwrap();
        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(url) = base_url.join(href) {
                    assets.push(Asset::new(url, "".into(), AssetType::Css, 0));
                }
            }
        }

        Ok(assets)
    }
}
