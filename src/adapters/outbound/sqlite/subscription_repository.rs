use anyhow::Context;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::{PlanId, Subscription, SubscriptionId, TenantId};
use crate::ports::SubscriptionRepository;

#[derive(Clone)]
pub struct SqliteSubscriptionRepository {
    pool: SqlitePool,
}

impl SqliteSubscriptionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl SubscriptionRepository for SqliteSubscriptionRepository {
    async fn insert_subscription(
        &self,
        tenant_id: &TenantId,
        plan_id: &PlanId,
    ) -> Result<Subscription, anyhow::Error> {
        let id = Uuid::new_v4().to_string();
        let tenant_id_str = tenant_id.as_ref();
        let plan_id_str = plan_id.as_ref();

        sqlx::query!(
            "INSERT INTO subscriptions (id, tenant_id, plan_id, created_at) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
            id,
            tenant_id_str,
            plan_id_str
        )
        .execute(&self.pool)
        .await
        .context("failed to insert subscription into database")?;

        Ok(Subscription::new(
            SubscriptionId::new(id),
            tenant_id.clone(),
            plan_id.clone(),
        ))
    }
}
