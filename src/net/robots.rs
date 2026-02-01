use robots_txt::{Robots, SimpleMatcher};
use url::Url;
use crate::types::error::{ArchivError, Result};
use crate::net::HttpClient;
use std::collections::HashMap;

pub struct RobotsHandler {
    cache: HashMap<String, String>, // host -> robots.txt content
}

impl RobotsHandler {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub async fn is_allowed(&mut self, url: &Url, user_agent: &str, client: &dyn HttpClient) -> Result<bool> {
        let host = url.host_str().ok_or_else(|| ArchivError::Url(url::ParseError::EmptyHost))?;
        let scheme = url.scheme();
        let robots_url = Url::parse(&format!("{}://{}/robots.txt", scheme, host))?;

        let content = if let Some(c) = self.cache.get(host) {
            c.clone()
        } else {
            match client.fetch(&robots_url).await {
                Ok(resp) => {
                    let c = String::from_utf8_lossy(&resp.content).to_string();
                    self.cache.insert(host.to_string(), c.clone());
                    c
                }
                Err(_) => {
                    // If robots.txt fails (e.g. 404), assume everything is allowed
                    return Ok(true);
                }
            }
        };

        let robots = Robots::from_str(&content);
        let section = robots.choose_section(user_agent);
        let matcher = SimpleMatcher::new(&section.rules);
        Ok(matcher.check_path(url.path()))
    }
}
