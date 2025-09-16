mod models;

use axum::{http::Method, middleware, routing::{get, post, delete}, Router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    init_tracing()?;
}
