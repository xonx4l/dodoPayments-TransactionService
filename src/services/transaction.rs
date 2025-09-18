use crate::{
    database::Database,
    error::{AppError, Result},
    models::{
        CreateTransactionRequest, Transaction, TransactionResponse, TransactionType,
    },
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionService {
    database: Arc<Database>,
}

impl TransactionService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub async fn create_transaction(
        &self,
        account_id: Uuid,
        req: CreateTransactionRequest,
     ) -> Result<TransactionResponse> {
        let span = tracing::info_span!(
            "create_transaction",
            account_id = %account_id,
            transaction_type = %req.r#type,
            amount = req.amount,
            idempotency_key = ?req.idempotency_key
        );
        let _enter = span.enter();

        tracing::info!("Creating transaction");

        let transaction_type = match req.r#type.as_str() {
            "credit" => TransactionType::Credit,
            "debit" => TransactionType::Debit,
            "transfer" => TransactionType::Transfer,
            _ => return Err(AppError::Internal(anyhow::anyhow!("Invalid transaction type"))),
        };

        if let Some(ref key) = req.idempotency_key {
            if let Some(existing) = self.get_transaction_by_idempotency_key(key).await? {
                return Ok(TransactionResponse {
                    transaction: existing,
                });
            }
        }

        let mut tx = self.database.begin_transaction().await?;

        let current_balance = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT balance
            FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::AccountNotFound {
            account_id: account_id.to_string(),
        })?;

        if transaction_type == TransactionType::Transfer {
           if req.counterparty_account_id.is_none() {
                return Err(AppError::Internal(anyhow::anyhow!("Missing counterparty account for transfer")));
            }
        }

        if matches!(transaction_type, TransactionType::Debit | TransactionType::Transfer) {
            if current_balance < req.amount {
                return Err(AppError::InsufficientFunds {
                    account_id: account_id.to_string(),
                    balance: current_balance,
                    required: req.amount,
                });
            }
        }

        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (account_id, counterparty_account_id, type, amount, description, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, account_id, counterparty_account_id, type, amount, description, status, idempotency_key, created_at, updated_at
            "#,
        )
        .bind(account_id)
        .bind(req.counterparty_account_id)
        .bind(transaction_type.to_string())
        .bind(req.amount)
        .bind(&req.description)
        .bind(&req.idempotency_key)
        .fetch_one(&mut *tx)
        .await?;


        let new_balance = match transaction_type {
            TransactionType::Credit => current_balance + req.amount,
            TransactionType::Debit => current_balance - req.amount,
            TransactionType::Transfer => {
                if let Some(counterparty_id) = req.counterparty_account_id {
                    let counterparty_balance = sqlx::query_scalar::<_, i64>(
                        r#"
                        SELECT balance
                        FROM accounts
                        WHERE id = $1
                        "#,
                    )
                    .bind(counterparty_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .ok_or_else(|| AppError::AccountNotFound {
                        account_id: counterparty_id.to_string(),
                    })?;


                    sqlx::query(
                        r#"
                        UPDATE accounts
                        SET balance = $1
                        WHERE id = $2
                        "#,
                    )
                    .bind(counterparty_balance + req.amount)
                    .bind(counterparty_id)
                    .execute(&mut *tx)
                    .await?;

                    current_balance - req.amount
                } else {
                    return Err(AppError::Internal(anyhow::anyhow!("Missing counterparty account for transfer")));
                }
            }
        };

        sqlx::query(
            r#"
            UPDATE accounts
            SET balance = $1
            WHERE id = $2
            "#,
        )
        .bind(new_balance)
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

       
        sqlx::query(
            r#"
            UPDATE transactions
            SET status = 'completed'
            WHERE id = $1
            "#,
        )
        .bind(transaction.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let completed_transaction = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT id, account_id, counterparty_account_id, type, amount, description, status, idempotency_key, created_at, updated_at
            FROM transactions
            WHERE id = $1
            "#,
        )
        .bind(transaction.id)
        .fetch_one(&*self.database.pool())
        .await?;

        tracing::info!(
            transaction_id = %completed_transaction.id,
            account_id = %completed_transaction.account_id,
            transaction_type = %completed_transaction.r#type,
            amount = completed_transaction.amount,
            new_balance = new_balance,
            "Transaction completed successfully"
        );

        crate::metrics::record_transaction_created(
            &completed_transaction.r#type,
            completed_transaction.amount as f64,
        );
        crate::metrics::record_balance_change(
            &completed_transaction.account_id.to_string(),
            current_balance as f64,
            new_balance as f64,
        );

        Ok(TransactionResponse {
            transaction: completed_transaction,
        })
    }

    pub async fn get_transaction(&self, transaction_id: Uuid) -> Result<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT id, account_id, counterparty_account_id, type, amount, description, status, idempotency_key, created_at, updated_at
            FROM transactions
            WHERE id = $1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(&*self.database.pool())
        .await?
        .ok_or_else(|| AppError::TransactionNotFound {
            transaction_id: transaction_id.to_string(),
        })?;

        Ok(transaction)
    }

    async fn get_transaction_by_idempotency_key(&self, key: &str) -> Result<Option<Transaction>> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT id, account_id, counterparty_account_id, type, amount, description, status, idempotency_key, created_at, updated_at
            FROM transactions
            WHERE idempotency_key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&*self.database.pool())
        .await?;

        Ok(transaction)
     }
}
