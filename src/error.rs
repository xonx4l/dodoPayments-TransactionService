use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum AppError {
     #[error("Database error: {0}")]
     Database(#[from] sqlx::Error),

     #[error("Validation error: {0}")]
     Validation(#[from] validator::ValidationErrors),

     #[error("Account not found: {account_id}")]
     AccountNotFound { account_id: String },

     #[error("Insufficient funds: account {account_id} has balance {balance}, required {required}")]
     InsufficientFunds{ account_id: String, balance: i64, required: i64},

     #[error("Transaction not found: {transaction_id}")]
     TransactionNotFound { transaction_id: String},

     #[error("Invalid API key")]
     InvalidApiKey,

     #[error("Webhook not found: {webhook_id}")]
     WebhookNotFound {webhook_id: String},

     #[error("Webhook delivery failed: {0}")]
     WebhookDeliveryFailed(String),

     #[error("Idempotency key already used: {Key}")]
     IdempotencyKeyUsed {Key: String},

     #[error("Rate limit exceeded")]
     RateLimitExceeded,

     #[error("Internal server error: {0}")]
     Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        
    }
}