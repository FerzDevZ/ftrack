use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leak {
    pub kind: String,
    pub value: String,
    pub source_url: String,
    pub context: String,
}
