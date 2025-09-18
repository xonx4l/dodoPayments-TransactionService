use axum::{
    extract::{Path, State},
    response::Json,
};
use uuid::Uuid;

use crate::{
    error::Result,
    models::{CreateWebhookRequest, WebhookResponse},
    services::{AccountService, TransactionService, WebhookService},
};

pub async fn register_webhook(
    State((_, _, webhook_service)): State<(AccountService, TransactionService, WebhookService)>,
    axum::extract::Extension(account_id): axum::extract::Extension<Uuid>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<WebhookResponse>> {
    let response = webhook_service.create_webhook(account_id, req).await?;
    Ok(Json(response))
}

pub async fn get_webhook(
    State((_, _, webhook_service)): State<(AccountService, TransactionService, WebhookService)>,
    Path(webhook_id): Path<Uuid>,
) -> Result<Json<WebhookResponse>> {
    let webhook = webhook_service.get_webhook(webhook_id).await?;
    Ok(Json(WebhookResponse { webhook }))
}

pub async fn update_webhook(
    State((_, _, webhook_service)): State<(AccountService, TransactionService, WebhookService)>,
    Path(webhook_id): Path<Uuid>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<WebhookResponse>> {
    let response = webhook_service.update_webhook(webhook_id, req).await?;
    Ok(Json(response))
}

pub async fn delete_webhook(
    State((_, _, webhook_service)): State<(AccountService, TransactionService, WebhookService)>,
    Path(webhook_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    webhook_service.delete_webhook(webhook_id).await?;
    Ok(Json(serde_json::json!({
        "message": "Webhook deleted successfully"
    })))
}