use anyhow::Context;
use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::TenantId;
use crate::ports::BillingProfileRepository;

#[derive(Clone)]
pub struct SqliteBillingProfileRepository {
    pool: SqlitePool,
}

impl SqliteBillingProfileRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BillingProfileRepository for SqliteBillingProfileRepository {
    async fn has_active_payment_method(&self, tenant_id: &TenantId) -> Result<bool, anyhow::Error> {
        let row = sqlx::query_as::<_, (bool,)>(
            "SELECT has_active_payment_method FROM billing_profiles WHERE tenant_id = ?1",
        )
        .bind(tenant_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch billing profile from database")?;

        Ok(row.map(|(has_payment,)| has_payment).unwrap_or(false))
    }
}
