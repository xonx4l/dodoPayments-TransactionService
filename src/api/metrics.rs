use axum::{
    http::StatusCode,
    response::IntoResponse,
};

pub async fn metrics_handler() -> impl IntoResponse {
    let accounts_created = crate::metrics::ACCOUNTS_CREATED
        .get()
        .map(|c| c.load(std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(0);
    
    let transactions_created = crate::metrics::TRANSACTIONS_CREATED
        .get()
        .map(|c| c.load(std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(0);
    
    let webhooks_delivered = crate::metrics::WEBHOOKS_DELIVERED
        .get()
        .map(|c| c.load(std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(0);

    let metrics = format!(
        "# HELP accounts_created_total Total number of accounts created\n\
         # TYPE accounts_created_total counter\n\
         accounts_created_total {}\n\
         \n\
         # HELP transactions_created_total Total number of transactions created\n\
         # TYPE transactions_created_total counter\n\
         transactions_created_total {}\n\
         \n\
         # HELP webhooks_delivered_total Total number of webhooks delivered\n\
         # TYPE webhooks_delivered_total counter\n\
         webhooks_delivered_total {}\n",
        accounts_created, transactions_created, webhooks_delivered
    );

    (StatusCode::OK, metrics)
}