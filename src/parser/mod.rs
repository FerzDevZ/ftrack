use crate::types::asset::Asset;
use crate::types::error::Result;
use url::Url;

pub trait ContentParser: Send + Sync {
    fn extract_links(&self, content: &str, base_url: &Url) -> Result<Vec<Url>>;
    fn extract_assets(&self, content: &str, base_url: &Url) -> Result<Vec<Asset>>;
}

pub mod html;
pub mod css;
pub mod js;
