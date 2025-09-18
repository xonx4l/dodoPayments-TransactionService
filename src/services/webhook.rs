use crate::{
    database::Database,
    error::{AppError, Result},
    models::{
        CreateWebhookRequest, Transaction, Webhook, WebhookPayload, WebhookResponse,
    },
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::json;
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct WebhookService {
    database: Arc<Database>,
    client: Client,
}

impl WebhookService {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            client: Client::new(),
        }
    }

    pub async fn create_webhook(
        &self,
        account_id: Uuid,
        req: CreateWebhookRequest,
    ) -> Result<WebhookResponse> {
        let span = tracing::info_span!(
            "create_webhook",
            account_id = %account_id,
            url = %req.url
        );
        let _enter = span.enter();
        
        tracing::info!("Creating webhook");
        
        let webhook = sqlx::query_as::<_, Webhook>(
            r#"
            INSERT INTO webhooks (account_id, url, events, secret)
            VALUES ($1, $2, $3, $4)
            RETURNING id, account_id, url, events, secret, is_active, created_at, updated_at
            "#,
        )
        .bind(account_id)
        .bind(&req.url)
        .bind(&req.events)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(&*self.database.pool())
        .await?;

        Ok(WebhookResponse { webhook })
    }

    pub async fn get_webhook(&self, webhook_id: Uuid) -> Result<Webhook> {
        let webhook = sqlx::query_as::<_, Webhook>(
            r#"
            SELECT id, account_id, url, events, secret, is_active, created_at, updated_at
            FROM webhooks
            WHERE id = $1
            "#,
        )
        .bind(webhook_id)
        .fetch_optional(&*self.database.pool())
        .await?
        .ok_or_else(|| AppError::WebhookNotFound {
            webhook_id: webhook_id.to_string(),
        })?;

        Ok(webhook)
    }

    pub async fn update_webhook(
        &self,
        webhook_id: Uuid,
        req: CreateWebhookRequest,
    ) -> Result<WebhookResponse> {
        let webhook = sqlx::query_as::<_, Webhook>(
            r#"
            UPDATE webhooks
            SET url = $1, events = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, account_id, url, events, secret, is_active, created_at, updated_at
            "#,
        )
        .bind(&req.url)
        .bind(&req.events)
        .bind(webhook_id)
        .fetch_optional(&*self.database.pool())
        .await?
        .ok_or_else(|| AppError::WebhookNotFound {
            webhook_id: webhook_id.to_string(),
        })?;

        Ok(WebhookResponse { webhook })
    }

    pub async fn delete_webhook(&self, webhook_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM webhooks
            WHERE id = $1
            "#,
        )
        .bind(webhook_id)
        .execute(&*self.database.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::WebhookNotFound {
                webhook_id: webhook_id.to_string(),
            });
        }

        Ok(())
    }

    pub async fn deliver_webhook(&self, transaction: &Transaction) -> Result<()> {
        let webhooks = sqlx::query_as::<_, Webhook>(
            r#"
            SELECT id, account_id, url, events, secret, is_active, created_at, updated_at
            FROM webhooks
            WHERE account_id = $1 AND is_active = true
            "#,
        )
        .bind(transaction.account_id)
        .fetch_all(&*self.database.pool())
        .await?;

        for webhook in webhooks {
            let event_type = match transaction.r#type.as_str() {
                "credit" => "transaction.credit",
                "debit" => "transaction.debit",
                "transfer" => "transaction.transfer",
                _ => continue,
            };

            if !webhook.events.contains(&event_type.to_string()) {
                continue;
            }

            let delivery_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO webhook_deliveries (webhook_id, transaction_id, max_attempts)
                VALUES ($1, $2, $3)
                RETURNING id
                "#,
            )
            .bind(webhook.id)
            .bind(transaction.id)
            .bind(3)
            .fetch_one(&*self.database.pool())
            .await?;

            self.deliver_webhook_async(webhook, transaction, delivery_id).await;
        }

        Ok(())
    }

    async fn deliver_webhook_async(
        &self,
        webhook: Webhook,
        transaction: &Transaction,
        delivery_id: Uuid,
    ) {
        let payload = WebhookPayload {
            event: match transaction.r#type.as_str() {
                "credit" => "transaction.credit",
                "debit" => "transaction.debit",
                "transfer" => "transaction.transfer",
                _ => "transaction.unknown",
            }
            .to_string(),
            transaction: transaction.clone(),
            timestamp: Utc::now(),
            signature: self.generate_signature(&webhook.secret, &json!(transaction).to_string()),
        };

        let response = self
            .client
            .post(&webhook.url)
            .json(&payload)
            .header("X-Webhook-Signature", &payload.signature)
            .header("X-Webhook-Event", &payload.event)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16() as i32;
                let body = resp.text().await.unwrap_or_default();

                sqlx::query(
                    r#"
                    UPDATE webhook_deliveries
                    SET status = 'delivered', response_status = $1, response_body = $2, attempts = attempts + 1, updated_at = NOW()
                    WHERE id = $3
                    "#,
                )
                .bind(status)
                .bind(body)
                .bind(delivery_id)
                .execute(&*self.database.pool())
                .await
                .ok();
            }
            Err(_) => {
                let next_retry = Utc::now() + chrono::Duration::minutes(5);

                sqlx::query(
                    r#"
                    UPDATE webhook_deliveries
                    SET status = 'retrying', attempts = attempts + 1, next_retry_at = $1, updated_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(next_retry)
                .bind(delivery_id)
                .execute(&*self.database.pool())
                .await
                .ok();
            }
        }
    }

    fn generate_signature(&self, secret: &str, payload: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    pub async fn retry_failed_deliveries(&self) -> Result<()> {
        #[derive(sqlx::FromRow)]
        struct DeliveryRow {
            id: Uuid,
            webhook_id: Uuid,
            transaction_id: Uuid,
        }

        let deliveries = sqlx::query_as::<_, DeliveryRow>(
            r#"
            SELECT wd.id, wd.webhook_id, wd.transaction_id
            FROM webhook_deliveries wd
            JOIN webhooks w ON wd.webhook_id = w.id
            WHERE wd.status = 'retrying' 
            AND wd.next_retry_at <= NOW()
            AND wd.attempts < wd.max_attempts
            AND w.is_active = true
            "#,
        )
        .fetch_all(&*self.database.pool())
        .await?;

        for delivery in deliveries {
            let webhook = self.get_webhook(delivery.webhook_id).await?;
            let transaction = sqlx::query_as::<_, Transaction>(
                r#"
                SELECT id, account_id, counterparty_account_id, type, amount, description, status, idempotency_key, created_at, updated_at
                FROM transactions
                WHERE id = $1
                "#,
            )
            .bind(delivery.transaction_id)
            .fetch_one(&*self.database.pool())
            .await?;

            self.deliver_webhook_async(webhook, &transaction, delivery.id)
                .await;
        }

        Ok(())
    }
}