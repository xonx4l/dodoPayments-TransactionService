use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

pub static ACCOUNTS_CREATED: OnceLock<AtomicU64> = OnceLock::new();
pub static TRANSACTIONS_CREATED: OnceLock<AtomicU64> = OnceLock::new();
pub static WEBHOOKS_DELIVERED: OnceLock<AtomicU64> = OnceLock::new();

pub fn init_metrics() -> anyhow::Result<()> {
    ACCOUNTS_CREATED.set(AtomicU64::new(0)).map_err(|_| anyhow::anyhow!("Failed to set accounts_created counter"))?;
    TRANSACTIONS_CREATED.set(AtomicU64::new(0)).map_err(|_| anyhow::anyhow!("Failed to set transactions_created counter"))?;
    WEBHOOKS_DELIVERED.set(AtomicU64::new(0)).map_err(|_| anyhow::anyhow!("Failed to set webhooks_delivered counter"))?;

    tracing::info!("Metrics initialized");
    Ok(())
}

pub fn record_account_created() {
    if let Some(counter) = ACCOUNTS_CREATED.get() {
        counter.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_transaction_created(transaction_type: &str, amount: f64) {
    if let Some(counter) = TRANSACTIONS_CREATED.get() {
        counter.fetch_add(1, Ordering::Relaxed);
    }
    tracing::info!(
        transaction_type = %transaction_type,
        amount = amount,
        "Transaction metrics recorded"
    );
}

pub fn record_balance_change(account_id: &str, old_balance: f64, new_balance: f64) {
    let change = new_balance - old_balance;
    tracing::info!(
        account_id = %account_id,
        old_balance = old_balance,
        new_balance = new_balance,
        change = change,
        "Balance change metrics recorded"
    );
}

pub fn record_webhook_delivered(success: bool, duration_seconds: f64) {
    if let Some(counter) = WEBHOOKS_DELIVERED.get() {
        counter.fetch_add(1, Ordering::Relaxed);
    }
    tracing::info!(
        success = success,
        duration_seconds = duration_seconds,
        "Webhook delivery metrics recorded"
    );
}