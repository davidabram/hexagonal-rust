use anyhow::Context;
use sqlx::SqlitePool;
use tracing::{error, instrument};

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
    #[instrument(
        name = "has_active_payment_method",
        skip(self),
        fields(db.system = "sqlite", tenant_id = %tenant_id)
    )]
    async fn has_active_payment_method(&self, tenant_id: &TenantId) -> Result<bool, anyhow::Error> {
        let tenant_id_str = tenant_id.as_ref();
        let row = sqlx::query_as!(
            BillingProfileRow,
            "SELECT has_active_payment_method FROM billing_profiles WHERE tenant_id = ?1",
            tenant_id_str
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch billing profile from database")
        .inspect_err(|e| {
            error!(error = %e, tenant_id = %tenant_id, "billing profile query failed");
        })?;

        let has_payment = row.map(|r| r.has_active_payment_method).unwrap_or(false);

        Ok(has_payment)
    }
}
