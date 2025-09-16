use chrono::{DateTime,Utc};
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

#[derive(Debug ,Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction{
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
        match self{
             TransactionType::Credit => write!(f,"credit"),
             TransactionType::Debit => write!(f,"debit"),
             TransactionType::Transfer => write!(f,"transfer"),
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
    fn fmt(&mut self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
             TransactionStatus::Pending => write!(f,"pending"),
             TransactionStatus::Completed => write!(f,"Completed"),
             TransactionStatus::Failed => write!(f,"failed"),
             TransactionStatus::Cancelled => write!(f,"cancelled"),
        }
    }
}