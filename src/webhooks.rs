use crate::services::WebhookService;
use std::sync::Arc;
use tokio_cron_scheduler::{Job , JobScheduler};

pub async fn start_webhook_retry_scheduler(webhook_service: Arc<WebhookService>) -> anyhow::Result<()> {
    let sched = JobScheduler::new().await?;

    sched
       .add(
           Job::new_async("0 */5 * * * *", move |_uuid, _l| {
               let webhook_service = webhook_service.clone();
               Box::pin(async move {
                    if let Err(e) = webhook_service.retry_failed_deliveries().await {
                        tracing::error!("Failed to retry webhook deliveries: {}", e);
                    }
               })
           })
           .unwrap(),
        )
        .await?;

    sched.start().await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}