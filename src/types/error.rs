use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ArchivError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Database error: {0}")]
    Database(String),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Robots.txt violation: {0}")]
    RobotsViolation(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Browser error: {0}")]
    Browser(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, ArchivError>;

impl From<tokio_rusqlite::Error> for ArchivError {
    fn from(err: tokio_rusqlite::Error) -> Self {
        ArchivError::Database(err.to_string())
    }
}
