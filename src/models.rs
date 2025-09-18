use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub business_name: String,
    pub email: String,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub account_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub counterparty_account_id: Option<Uuid>,
    pub r#type: String,
    pub amount: i64,
    pub description: Option<String>,
    pub status: String,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Credit,
    Debit,
    Transfer,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Credit => write!(f, "credit"),
            TransactionType::Debit => write!(f, "debit"),
            TransactionType::Transfer => write!(f, "transfer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Completed => write!(f, "completed"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub account_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub transaction_id: Uuid,
    pub status: WebhookDeliveryStatus,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "webhook_delivery_status", rename_all = "lowercase")]
pub enum WebhookDeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
}


#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountRequest {
    #[validate(length(min = 1, max = 255))]
    pub business_name: String,
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    pub account: Account,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub account: Account,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub account_id: Uuid,
    pub balance: i64,
    pub currency: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTransactionRequest {
    #[validate(length(min = 1, max = 255))]
    pub idempotency_key: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub r#type: String,
    #[validate(range(min = 1))]
    pub amount: i64,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub counterparty_account_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWebhookRequest {
    #[validate(url)]
    pub url: String,
    #[validate(length(min = 1))]
    pub events: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub webhook: Webhook,
}

#[derive(Debug, Serialize)]
pub struct WebhookDeliveryResponse {
    pub webhook_delivery: WebhookDelivery,
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub transaction: Transaction,
    pub timestamp: DateTime<Utc>,
    pub signature: String,
}