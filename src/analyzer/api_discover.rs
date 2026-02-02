use crate::types::endpoint::Endpoint;
use regex::Regex;
use super::ApiDiscovery;
use super::signatures::SignatureSet;

pub struct DefaultApiDiscoverer {
    signatures: Vec<crate::analyzer::signatures::Signature>,
}

impl DefaultApiDiscoverer {
    pub fn new() -> Self {
        // Coba load dari direktori konfigurasi
        match SignatureSet::load_directory("config/signatures") {
            Ok(set) => {
                let api_sigs = set.signatures.into_iter()
                    .filter(|s| s.kind == "api")
                    .collect();
                Self { signatures: api_sigs }
            }
            Err(_) => {
                // Fallback jika folder tidak ada
                Self { signatures: Vec::new() }
            }
        }
    }
}

impl ApiDiscovery for DefaultApiDiscoverer {
    fn discover(&self, source_url: &str, content: &str) -> Vec<Endpoint> {
        let mut endpoints = Vec::new();
        
        for sig in &self.signatures {
            if let Ok(re) = Regex::new(&sig.pattern) {
                for mat in re.find_iter(content) {
                    let found = mat.as_str().to_string();
                    
                    // Ambil konteks (30 karakter sebelum/sesudah)
                    let start = mat.start().saturating_sub(30);
                    let end = (mat.end() + 30).min(content.len());
                    let context = content[start..end].replace('\n', " ").trim().to_string();

                    endpoints.push(Endpoint {
                        method: "DISCOVERED".to_string(),
                        url: found,
                        source_url: source_url.to_string(),
                        context,
                    });
                }
            }
        }
        
        endpoints.dedup_by(|a, b| a.url == b.url);
        endpoints
    }
}
