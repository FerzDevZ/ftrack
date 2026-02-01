use std::path::Path;
use tokio_rusqlite::Connection;
use url::Url;
use crate::types::asset::{Asset, AssetType};
use crate::types::error::Result;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path).await?;
        
        // Initialize schema
        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS queue (
                    url TEXT PRIMARY KEY,
                    asset_type TEXT,
                    depth INTEGER,
                    priority INTEGER
                )",
                [],
            )?;
            
            conn.execute(
                "CREATE TABLE IF NOT EXISTS visited (
                    url TEXT PRIMARY KEY,
                    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;
            
            conn.execute(
                "CREATE TABLE IF NOT EXISTS tech_stack (
                    url TEXT,
                    tech TEXT,
                    PRIMARY KEY (url, tech)
                )",
                [],
            )?;
            
            conn.execute(
                "CREATE TABLE IF NOT EXISTS leaks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    kind TEXT,
                    value TEXT,
                    url TEXT,
                    context TEXT,
                    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS endpoints (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    method TEXT,
                    url TEXT,
                    source_url TEXT,
                    context TEXT,
                    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;
            
            Ok(())
        }).await?;

        Ok(Self { conn })
    }

    pub async fn push_queue(&self, asset: Asset) -> Result<()> {
        let url = asset.url.to_string();
        let asset_type = serde_json::to_string(&asset.asset_type).unwrap_or_else(|_| "Html".into());
        let depth = asset.depth as i64;
        let priority = asset.priority;

        self.conn.call(move |conn| {
            conn.execute(
                "INSERT OR IGNORE INTO queue (url, asset_type, depth, priority) VALUES (?1, ?2, ?3, ?4)",
                (url, asset_type, depth, priority),
            )?;
            Ok(())
        }).await?;
        
        Ok(())
    }

    pub async fn pop_queue(&self) -> Result<Option<Asset>> {
        let res = self.conn.call(|conn| {
            let mut stmt = conn.prepare("SELECT url, asset_type, depth, priority FROM queue ORDER BY priority DESC LIMIT 1")?;
            let mut rows = stmt.query([])?;
            
            if let Some(row) = rows.next()? {
                let url_str: String = row.get(0)?;
                let asset_type_str: String = row.get(1)?;
                let depth: i64 = row.get(2)?;
                let priority: i32 = row.get(3)?;
                
                let url = Url::parse(&url_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let asset_type: AssetType = serde_json::from_str(&asset_type_str).unwrap_or(AssetType::Html);
                
                conn.execute("DELETE FROM queue WHERE url = ?1", [url_str])?;
                
                let mut asset = Asset::new(url, "".into(), asset_type, depth as usize);
                asset.priority = priority;
                Ok(Some(asset))
            } else {
                Ok(None)
            }
        }).await?;
        Ok(res)
    }

    pub async fn mark_visited(&self, url: &Url) -> Result<()> {
        let url_str = url.to_string();
        self.conn.call(move |conn| {
            conn.execute("INSERT OR IGNORE INTO visited (url) VALUES (?1)", [url_str])?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn is_visited(&self, url: &Url) -> Result<bool> {
        let url_str = url.to_string();
        let res = self.conn.call(move |conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM visited WHERE url = ?1",
                [url_str],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        }).await?;
        Ok(res)
    }

    pub async fn save_tech_stack(&self, url: &Url, tech_stack: Vec<String>) -> Result<()> {
        let url_str = url.to_string();
        self.conn.call(move |conn| {
            let tx = conn.transaction()?;
            for tech in tech_stack {
                tx.execute(
                    "INSERT OR IGNORE INTO tech_stack (url, tech) VALUES (?1, ?2)",
                    (&url_str, tech),
                )?;
            }
            tx.commit()?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn save_leak(&self, leak: crate::types::leak::Leak) -> Result<()> {
        let kind = leak.kind;
        let value = leak.value;
        let url = leak.url;
        let context = leak.context;

        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO leaks (kind, value, url, context) VALUES (?1, ?2, ?3, ?4)",
                (kind, value, url, context),
            )?;
            Ok(())
        }).await?;
        
        Ok(())
    }

    pub async fn save_endpoint(&self, endpoint: crate::types::endpoint::Endpoint) -> Result<()> {
        let method = endpoint.method;
        let url = endpoint.url;
        let source_url = endpoint.source_url;
        let context = endpoint.context;

        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO endpoints (method, url, source_url, context) VALUES (?1, ?2, ?3, ?4)",
                (method, url, source_url, context),
            )?;
            Ok(())
        }).await?;
        
        Ok(())
    }
}
