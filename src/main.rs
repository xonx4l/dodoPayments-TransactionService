mod api;
mod config;
mod database;
mod error;
mod metrics;
mod models;
mod services;
mod webhooks;

use axum::{
    http::Method,
    middleware,
    routing::{get, post, delete},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber;

use crate::{
    api::{
        accounts, auth, health, metrics as api_metrics, transactions, webhooks as webhook_routes,
    },
    config::Config,
    database::Database,
    services::{AccountService, TransactionService, WebhookService},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;

    crate::metrics::init_metrics()?;

    let config = Config::load()?;
    
    let database = Database::new(&config.database_url).await?;
    database.migrate().await?;

    let database = Arc::new(database);
    let account_service = AccountService::new(database.clone());
    let transaction_service = TransactionService::new(database.clone());
    let webhook_service = WebhookService::new(database.clone());

    let app = Router::new()
        .route("/health", get(health::health_check))
        .route("/metrics", get(api_metrics::metrics_handler))
        .route("/api/v1/accounts", post(accounts::create_account))
        .nest("/api/v1", 
            Router::new()
                .route("/accounts/:account_id", get(accounts::get_account))
                .route("/accounts/:account_id/balance", get(accounts::get_balance))
                .route("/transactions", post(transactions::create_transaction))
                .route("/transactions/:transaction_id", get(transactions::get_transaction))
                .route("/webhooks", post(webhook_routes::register_webhook))
                .route("/webhooks/:webhook_id", get(webhook_routes::get_webhook))
                .route("/webhooks/:webhook_id", post(webhook_routes::update_webhook))
                .route("/webhooks/:webhook_id", delete(webhook_routes::delete_webhook))
                .layer(middleware::from_fn_with_state(
                    (account_service.clone(), transaction_service.clone(), webhook_service.clone()),
                    auth::auth_middleware,
                ))
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                        .allow_headers(Any),
                ),
        )
        .with_state((account_service, transaction_service, webhook_service));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server starting on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn init_tracing() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "transaction_service=debug,tower_http=debug".into()),
        )
        .init();

    tracing::info!("Structured logging initialized");
    Ok(())
}
