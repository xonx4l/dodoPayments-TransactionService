use axum::{
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::services::{AccountService, TransactionService, WebhookService};

pub async fn auth_middleware(
    State((account_service, _transaction_service, _webhook_service)): State<(
        AccountService,
        TransactionService,
        WebhookService,
    )>,
    mut request: axum::http::Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let api_key = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let account_id = account_service
        .validate_api_key(api_key)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(account_id);

    Ok(next.run(request).await)
}