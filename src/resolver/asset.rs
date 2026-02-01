use crate::resolver::AssetResolver;
use url::Url;
use std::path::PathBuf;

pub struct DefaultAssetResolver {
    #[allow(dead_code)]
    base_dir: PathBuf,
}

impl DefaultAssetResolver {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl AssetResolver for DefaultAssetResolver {
    fn resolve_local_path(&self, url: &Url) -> PathBuf {
        let host = url.host_str().unwrap_or("unknown");
        let path = url.path().trim_start_matches('/');
        
        let mut local_path = PathBuf::from(host);
        if path.is_empty() {
            local_path.push("index.html");
        } else {
            local_path.push(path);
            // If it's a directory-like URL, append index.html
            if url.path().ends_with('/') {
                local_path.push("index.html");
            } else if !path.contains('.') {
                // Heuristic: if no extension, maybe it's a clean URL
                local_path.set_extension("html");
            }
        }
        local_path
    }
}
