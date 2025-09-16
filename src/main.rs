mod models;
mod database;

use axum::{http::Method, middleware, routing::{get, post, delete}, Router};

use crate::{
    database::Database;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    init_tracing()?;

    let database = Database::new().await?;

    let database = Arc::new(database);
}
