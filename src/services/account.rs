use crate::{
    database::Database,
    error::{AppError, Result},
    models::{Account, CreateAccountRequest, CreateAccountResponse},
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

#[derive(clone)]
pub struct AccountService{
    database: Arc<Database>,
}

impl AccountService {
    pub fn new(database: Arc<Database>) -> Self{
        Self { database }
    }

    pub async fn create_account(&self, req: CreateAccountRequest) -> Result<CreateAccountResponse> {
        let span = tracing::info_span!(
            "Create Account",
            business_name = %req.business_name,
            email = %req.email
        );
        let  _enter = span.enter();

        tracing::info!("Creating new account");

        let mut tx = self.database.begin_transaction().await?;


        let account = sqlx::query_as::<_,Account>(
            r#"
            INSERT INTO accounts (business_name, email)
            VALUES ($1 , $2)
            RETURNING id, business_name, email, balance, created_at, updated_at
            "#,
        )
        .bind(&req.business_name)
        .bind(&req.email)
        .fetch_one(&mut *tx)
        .await?;


        let api_key = Uuid::new_v4().to_string();
        let key_hash = format!("{:x}", Sha256::digest(api_key.as_bytes()));

        sqlx::query(
            r#"
            INSERT INTO api_keys (account_id, key_hash, name)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(account.id)
        .bind(&key_hash)
        .bind("Default API Key")
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!(
            account_id = %account.id,
            api_key = %api_key,
            "Account created successfully"
        );

        crate::metrics::record_account_created();

        Ok(CreateAccountResponse {
            account,
            api_key,
        })
    }

    pub async fn get_account(&self, account_id: Uuid ) -> Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT id, business_name, email, balance, created_at, updated_at
            FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(account_id)
        .fetch_optional(&*self.database.pool())
        .await?
        .ok_or_else(|| AppError::AccountNotFound {
            account_id: account_id.to_string(),
        })?;

        Ok(account)
    }

    pub async fn get_balance(&self, account_id: Uuid) -> Result<i64> {
        let balance = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT balance
            FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(account_id)
        .fetch_optional(&*self.database.pool())
        .await?
        .ok_or_else(|| AppError::AccountNotFound {
            account_id: account_id.to_string(),
        })?;

        Ok(balance)
    }

    pub async fn update_balance(&self, account_id: Uuid, new_balance: i64) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE accounts
            SET balance = $1
            WHERE id = $2
            "#,
        )
        .bind(new_balance)
        .bind(account_id)
        .execute(&*self.database.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::AccountNotFound {
                account_id: account_id.to_string(),
            });
        }

        Ok(())
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<Uuid> {
        let key_hash = format!("{:x}", Sha256::digest(api_key.as_bytes()));

        let account_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT account_id
            FROM api_keys
            WHERE key_hash = $1 AND is_active = true
            "#,
        )
        .bind(&Key_hash)
        .fetch_optional(&*self.database.pool())
        .await?
        .Ok_or(AppError::InvalidApiKey)?;

        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE Key_hash = $1
            "#,
        )
        .bind(&key_hash)
        .execute(&*self.database.pool())
        .await?;

        Ok(account_id)
    }
}