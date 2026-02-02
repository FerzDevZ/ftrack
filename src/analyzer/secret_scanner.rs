use crate::types::leak::Leak;
use regex::Regex;
use super::SecretScanner;
use super::signatures::SignatureSet;

pub struct DefaultSecretScanner {
    signatures: Vec<crate::analyzer::signatures::Signature>,
}

impl DefaultSecretScanner {
    pub fn new() -> Self {
        // Coba load dari direktori konfigurasi
        match SignatureSet::load_directory("config/signatures") {
            Ok(set) => {
                let secret_sigs = set.signatures.into_iter()
                    .filter(|s| s.kind == "secret")
                    .collect();
                Self { signatures: secret_sigs }
            }
            Err(_) => {
                // Fallback jika folder tidak ada (minimal)
                Self { signatures: Vec::new() }
            }
        }
    }
}

impl SecretScanner for DefaultSecretScanner {
    fn scan(&self, source_url: &str, content: &str) -> Vec<Leak> {
        let mut leaks = Vec::new();
        
        for sig in &self.signatures {
            if let Ok(re) = Regex::new(&sig.pattern) {
                for mat in re.find_iter(content) {
                    let found = mat.as_str().to_string();
                    
                    // Ambil konteks (30 karakter sebelum/sesudah)
                    let start = mat.start().saturating_sub(30);
                    let end = (mat.end() + 30).min(content.len());
                    let context = content[start..end].replace('\n', " ").trim().to_string();

                    leaks.push(Leak {
                        kind: sig.name.clone(),
                        value: found,
                        source_url: source_url.to_string(),
                        context,
                    });
                }
            }
        }
        
        leaks.dedup_by(|a, b| a.value == b.value);
        leaks
    }
}
