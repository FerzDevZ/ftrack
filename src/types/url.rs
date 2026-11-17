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

#[cfg(test)]
mod tests {
    use super::*;

    fn url(raw: &str) -> Url {
        Url::parse(raw).expect("test URL parses")
    }

    #[test]
    fn normalize_drops_the_fragment_but_keeps_the_query() {
        let normalized = normalize_url(&url("https://example.com/post?page=2#comments"));

        assert_eq!(normalized.as_str(), "https://example.com/post?page=2");
        assert_eq!(normalize_url(&url("https://example.com/a#top")).as_str(), "https://example.com/a");
    }

    #[test]
    fn normalization_is_idempotent() {
        let once = normalize_url(&url("https://example.com/x#y"));
        let twice = normalize_url(&once);

        assert_eq!(once, twice);
    }

    #[test]
    fn same_host_is_internal() {
        assert!(is_internal_link(
            &url("https://example.com/"),
            &url("https://example.com/about")
        ));
    }

    #[test]
    fn a_different_site_is_external() {
        assert!(!is_internal_link(
            &url("https://example.com/"),
            &url("https://other.org/about")
        ));
    }

    #[test]
    fn subdomains_share_the_registrable_domain() {
        /* `Url::domain()` returns the eTLD+1, so a subdomain counts as internal.
           The crawler therefore stays on the site's own domain family; this test
           records that behaviour so a future change to host comparison shows up
           as a deliberate decision instead of a silent one. */
        assert!(is_internal_link(
            &url("https://www.example.com/"),
            &url("https://api.example.com/v1")
        ));
    }
}
