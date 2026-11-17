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
                    /* `javascript:`, `mailto:` and `data:` resolve happily but are
                       not pages; queueing them spent a crawl slot and produced a
                       saved file named after the scheme. */
                    if matches!(url.scheme(), "http" | "https") {
                        links.push(url);
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://example.com/docs/start").expect("base URL parses")
    }

    #[test]
    fn links_resolve_against_the_page_url() {
        let html = r#"
            <a href="/about">about</a>
            <a href="next.html">next</a>
            <a href="https://other.org/x">external</a>
        "#;

        let links = HtmlParser.extract_links(html, &base()).expect("links parse");

        assert_eq!(
            links.iter().map(|u| u.as_str()).collect::<Vec<_>>(),
            vec![
                "https://example.com/about",
                "https://example.com/docs/next.html",
                "https://other.org/x",
            ]
        );
    }

    #[test]
    fn anchors_without_a_usable_href_are_skipped() {
        let html = r##"
            <a>no attribute</a>
            <a href="javascript:void(0)">javascript</a>
            <a href="mailto:hi@example.com">mail</a>
            <a href="#section">fragment only</a>
        "##;

        let links = HtmlParser.extract_links(html, &base()).expect("links parse");

        /* A bare fragment resolves to the page itself, so exactly one link survives. */
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].as_str(), "https://example.com/docs/start#section");
    }

    #[test]
    fn assets_keep_their_type_and_resolve_relative_paths() {
        let html = r#"
            <img src="../img/logo.png">
            <script src="/js/app.js"></script>
            <link rel="stylesheet" href="/css/site.css">
            <link rel="preload" href="/font.woff2">
        "#;

        let assets = HtmlParser.extract_assets(html, &base()).expect("assets parse");

        assert_eq!(assets.len(), 3, "a preload link is not a stylesheet");
        assert_eq!(assets[0].url.as_str(), "https://example.com/img/logo.png");
        assert!(matches!(assets[0].asset_type, AssetType::Image));
        assert_eq!(assets[1].url.as_str(), "https://example.com/js/app.js");
        assert!(matches!(assets[1].asset_type, AssetType::Js));
        assert_eq!(assets[2].url.as_str(), "https://example.com/css/site.css");
        assert!(matches!(assets[2].asset_type, AssetType::Css));
    }
}
