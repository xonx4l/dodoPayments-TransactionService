use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
}

impl config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = Config {
            port: env::var("PORT")
               .unwrap_or_else(|_| "3000".to_string())
               .parse()
               .unwrap_or(3000),
            database_url: env::var("DATABASE_URL")
               .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/transaction_service".to_string()),
        };

        Ok(config)
    }
}