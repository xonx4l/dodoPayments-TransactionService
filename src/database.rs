use anyhow::Result;
use sqlx::{PgPool,Postgres,Transaction};
use std::sync::Arc;

#[derive(Debug, Clone)] 
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self{
            pool: Arc::new(pool),
        })
   }

   pub fn pool(&self) -> &PgPool {
        &self.pool
   }
}