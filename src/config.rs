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
    }
}