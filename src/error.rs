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
        let(status, error_message) = match self {
            AppError::AccountNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::TransactionNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::WebhookNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InsufficientFunds { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Invalid API key".to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::IdempotencyKeyUsed { .. } => (StatusCode::CONFLICT, self.to_string()),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            AppError::WebhookDeliveryFailed(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;