use axum::{
    extract::{Path, State},
    response::Json,
};
use uuid::Uuid;

use crate::{
    error::Result,
    models::{CreateTransactionRequest, TransactionResponse},
    services::{AccountService, TransactionService, WebhookService},
};

pub async fn create_transaction(
    State((_account_service, transaction_service, webhook_service)): State<(
        AccountService,
        TransactionService,
        WebhookService,
    )>,
    axum::extract::Extension(account_id): axum::extract::Extension<Uuid>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<TransactionResponse>> {
    let response = transaction_service
        .create_transaction(account_id, req)
        .await?;

    let webhook_service_clone = webhook_service.clone();
    let transaction_clone = response.transaction.clone();
    tokio::spawn(async move {
        let _ = webhook_service_clone.deliver_webhook(&transaction_clone).await;
    });

    Ok(Json(response))
}

pub async fn get_transaction(
    State((_, transaction_service, _)): State<(AccountService, TransactionService, WebhookService)>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>> {
    let transaction = transaction_service.get_transaction(transaction_id).await?;
    Ok(Json(TransactionResponse { transaction }))
}
