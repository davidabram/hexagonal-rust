use anyhow::Context;
use sqlx::SqlitePool;

use crate::domain::TenantId;
use crate::ports::BillingProfileRepository;

struct BillingProfileRow {
    has_active_payment_method: bool,
}

#[derive(Clone)]
pub struct SqliteBillingProfileRepository {
    pool: SqlitePool,
}

impl SqliteBillingProfileRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl BillingProfileRepository for SqliteBillingProfileRepository {
    async fn has_active_payment_method(&self, tenant_id: &TenantId) -> Result<bool, anyhow::Error> {
        let tenant_id_str = tenant_id.as_ref();
        let row = sqlx::query_as!(
            BillingProfileRow,
            "SELECT has_active_payment_method FROM billing_profiles WHERE tenant_id = ?1",
            tenant_id_str
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch billing profile from database")?;

        Ok(row.map(|r| r.has_active_payment_method).unwrap_or(false))
    }
}
