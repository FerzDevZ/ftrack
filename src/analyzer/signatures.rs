use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub severity: String,
    pub description: String,
    #[serde(default)]
    pub kind: String, // e.g., "secret", "api"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignatureSet {
    pub signatures: Vec<Signature>,
}

impl SignatureSet {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Gagal membaca file signature: {:?}", path.as_ref()))?;
        let set: SignatureSet = serde_yaml::from_str(&content)
            .with_context(|| format!("Gagal mem-parse YAML signature: {:?}", path.as_ref()))?;
        Ok(set)
    }

    pub fn load_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let mut all_signatures = Vec::new();
        if !dir.as_ref().exists() {
            return Ok(Self::default());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let set = Self::from_file(&path)?;
                all_signatures.extend(set.signatures);
            }
        }
        Ok(Self { signatures: all_signatures })
    }
}
