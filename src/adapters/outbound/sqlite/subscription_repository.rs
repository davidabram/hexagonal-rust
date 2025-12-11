use anyhow::Context;
use async_trait::async_trait;
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

#[async_trait]
impl SubscriptionRepository for SqliteSubscriptionRepository {
    async fn insert_subscription(
        &self,
        tenant_id: &TenantId,
        plan_id: &PlanId,
    ) -> Result<Subscription, anyhow::Error> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO subscriptions (id, tenant_id, plan_id, created_at) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)"
        )
        .bind(&id)
        .bind(tenant_id.as_str())
        .bind(plan_id.as_str())
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
