#[allow(dead_code)]
use url::Url;

#[allow(dead_code)]
pub fn normalize_url(url: &Url) -> Url {
    let mut normalized = url.clone();
    normalized.set_fragment(None);
    // Add more normalization logic (e.g., stripping trailing slashes if appropriate)
    normalized
}

#[allow(dead_code)]
pub fn is_internal_link(base: &Url, target: &Url) -> bool {
    base.domain() == target.domain()
}
